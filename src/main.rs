use libloading::{Library, Symbol};
use std::os::raw::{c_char, c_void};
use std::ffi::CStr;
use std::fmt;
use rand::Rng;

//94 / 102
const OPCODE: [u8; 102] = [
  0x07,
  0x0f,
  0x17,
  0x1f,
  0x27,
  0x2f,
  0x37,
  0x3f,
  0x47,
  0x4f,
  0x57,
  0x5f,
  0x67,
  0x6f,
  0x77,
  0x7f,
  0x87,
  0x97,
  0x9f,
  0xa7,
  0xaf,
  0xb7,
  0xbf,
  0xc7,
  0xcf,
  0x04,
  0x0c,
  0x14,
  0x1c,
  0x24,
  0x2c,
  0x34,
  0x3c,
  0x44,
  0x4c,
  0x54,
  0x5c,
  0x64,
  0x6c,
  0x74,
  0x7c,
  0x84,
  0x94,
  0x9c,
  0xa4,
  0xac,
  0xb4,
  0xbc,
  0xc4,
  0xcc,
  0xd4,
  0xd4,
  0xd4,
  0xdc,
  0xdc,
  0xdc,
  0x18,
  0x20,
  0x28,
  0x30,
  0x38,
  0x40,
  0x48,
  0x50,
  0x58,
  0x61,
  0x69,
  0x71,
  0x79,
  0x62,
  0x6a,
  0x72,
  0x7a,
  0x63,
  0x6b,
  0x73,
  0x7b,
  0x05,
  0x15,
  0x1d,
  0x25,
  0x2d,
  0x35,
  0x3d,
  0xa5,
  0xad,
  0xb5,
  0xbd,
  0x45,
  0x4d,
  0x55,
  0x5d,
  0x65,
  0x6d,
  0x75,
  0x7d,
  0xc5,
  0xcd,
  0xd5,
  0xdd,
  0x85,
  0x95
];

const BRANCH_OPCODE: [u8; 23] = [
  0x05,
  0x15,
  0x1d,
  0x25,
  0x2d,
  0x35,
  0x3d,
  0xa5,
  0xad,
  0xb5,
  0xbd,
  0x45,
  0x4d,
  0x55,
  0x5d,
  0x65,
  0x6d,
  0x75,
  0x7d,
  0xc5,
  0xcd,
  0xd5,
  0xdd
];

const BITSWAP_INSTR: [u8; 6] = [
  0xd4,
  0xd4,
  0xd4,
  0xdc,
  0xdc,
  0xdc
];

#[derive(PartialEq, Clone, Copy)]
struct EbpfBytecode([c_char; 8]);

impl Default for EbpfBytecode {
  fn default() -> Self{
    Self([0; 8])
  }
}

impl fmt::Debug for EbpfBytecode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(f, "[")?;
      for (i, byte) in self.0.iter().enumerate() {
          write!(f, "0x{:02X}", byte)?;
          if i < self.0.len() - 1 {
              write!(f, ", ")?;
          }
      }
      write!(f, "]")
  }
}

impl From<EbpfInstr> for EbpfBytecode{
  fn from(instr: EbpfInstr) -> EbpfBytecode{
    let mut arr = [0 as c_char; 8];

    let instr_u64 : u64 = (instr.imm    as u64 & 0xFF_FF_FF_FF) << 32
                        | (instr.offset as u64 & 0xFF_FF) << 16
                        | (instr.src    as u64 & 0xF) << 8
                        | (instr.dst    as u64 & 0xF) << 12
                        | (instr.opcode as u64 & 0xFF);

    for i in 0..8{
      arr[i] = ((instr_u64 >> (8 * i)) & 0xFF) as c_char;
    }

    EbpfBytecode(arr)
  }
}

#[derive(PartialEq, Clone, Copy)]
struct EbpfInstr{
  opcode: u8,
  dst: u8,
  src: u8,
  offset: u16,
  imm: i32
}

impl EbpfInstr{
  fn new(opcode: u8, dst: u8, src: u8, offset: u16, imm: i32) -> Self{
    Self{
      opcode: opcode,
      dst: dst,
      src: src,
      offset: offset,
      imm: imm
    }
  }
  
  fn generate_random_instr (i: i32) -> EbpfInstr{
    let opcode = OPCODE[rand::thread_rng().gen_range(0..OPCODE.len())];

    let mut is_branch = false;
    for o in BRANCH_OPCODE.iter(){
      if *o == opcode{
        is_branch = true;
        break;
      } 
    }

    let mut is_bit_swap = false;
    for o in BITSWAP_INSTR.iter(){
      if *o == opcode{
        is_bit_swap = true;
        break;
      }
    }

    let mut offset = 0;
    if is_branch{
      offset = rand::thread_rng().gen_range(-i .. (((NBR_INSTR + 1) as i32) - i - 1)) as u16;
    }
    else{
      offset = rand::thread_rng().gen_range(0..0xFF_FF);
    }

    let mut imm = 0;
    if is_bit_swap{
      let values = [16, 32, 64];
      imm = values[rand::thread_rng().gen_range(0..3)];
    }
    else{
      imm = rand::thread_rng().gen::<i32>();
    }

    EbpfInstr{
      opcode: opcode,
      dst: rand::thread_rng().gen_range(0..10),
      src: rand::thread_rng().gen_range(0..10),
      offset: offset,
      imm: imm
    }
  }
}

impl fmt::Debug for EbpfInstr{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result{
    write!(f, "opcode: 0x{:02X?}, dst: 0x{:02X?}, src: 0x{:02X?}, offset: 0x{:04X?}, imm: 0x{:08X?}", self.opcode, self.dst, self.src, self.offset, self.imm)
  }
}

type UbpfVm = c_void;
type UbpfCreateFn = unsafe extern fn() -> *mut UbpfVm;
type UbpfLoadFn = unsafe extern fn(*mut UbpfVm, *const c_char, u32, *mut *const c_char) -> i32;
type UbpfCompileFn = unsafe extern fn(*mut UbpfVm, *mut *const c_char) -> u64;
type UbpfDeleteFn = unsafe extern fn(*mut UbpfVm) -> ();
type UbpfExecFn = unsafe extern fn(*mut UbpfVm, *mut i8, u32, *mut u64) -> i32;

fn get_functions_pointer() -> 
  Result<
    (
      Symbol<'static, UbpfCreateFn>, 
      Symbol<'static, UbpfLoadFn>, 
      Symbol<'static, UbpfCompileFn>, 
      Symbol<'static, UbpfDeleteFn>, 
      Symbol<'static, UbpfExecFn>
    ), 
    String
  > {
  unsafe{
    //link to libupf.so file
    let lib: &'static Library = Box::leak(Box::new(Library::new("../ubpf/vm/libubpf.so").unwrap()));

    let ubpf_create_fn: Symbol<'static, UbpfCreateFn> = lib.get(b"ubpf_create").unwrap();
    let ubpf_load_fn: Symbol<'static, UbpfLoadFn> = lib.get(b"ubpf_load").unwrap();
    let ubpf_compile_fn: Symbol<'static, UbpfCompileFn> = lib.get(b"ubpf_compile").unwrap();
    let ubpf_destroy_fn: Symbol<'static, UbpfDeleteFn> = lib.get(b"ubpf_destroy").unwrap();
    let ubpf_exec: Symbol<'static, UbpfExecFn> = lib.get(b"ubpf_exec").unwrap();

    Ok((ubpf_create_fn, ubpf_load_fn, ubpf_compile_fn, ubpf_destroy_fn, ubpf_exec))
  }
}

fn run_prgm (
  bytecode: &[EbpfBytecode], 
  symbols: &(
    Symbol<UbpfCreateFn>, 
    Symbol<UbpfLoadFn>, 
    Symbol<UbpfCompileFn>, 
    Symbol<UbpfDeleteFn>, 
    Symbol<UbpfExecFn>
  )) -> Result<u64, String> 
  {
  unsafe{
    let (
      ubpf_create_fn,
      ubpf_load_fn,
      _ubpf_compile_fn,
      ubpf_destroy_fn,
      ubpf_exec_fn
    ) = &symbols;

    let vm: *mut UbpfVm = ubpf_create_fn();

    if DEBUG{
      println!("[+]VM addr: {:?}", vm);
    }

    let mut errmsg_ptr: *const c_char = std::mem::MaybeUninit::<c_char>::uninit().as_mut_ptr();

    if ubpf_load_fn(vm, bytecode.as_ptr() as *const c_char, 8 * (NBR_INSTR + 1) as u32, &mut errmsg_ptr) < 0 {
      let errmsg = CStr::from_ptr(errmsg_ptr);
      ubpf_destroy_fn(vm);

      return Err(errmsg.to_string_lossy().to_string());
    }
    if DEBUG{
      println!("[+]FN Loaded");
    }

    let ret_val_ptr: *mut u64 = std::mem::MaybeUninit::<u64>::uninit().as_mut_ptr();

    const STACK_SIZE: usize = 8092;

    let mut buffer: Vec<u8> = vec![0; STACK_SIZE];
    let buf_ptr = buffer.as_mut_ptr() as *mut c_char;

    let exec_ret = ubpf_exec_fn(vm, buf_ptr, STACK_SIZE as u32, ret_val_ptr);

    if DEBUG{
      if exec_ret == 0{
        println!("[+]Exec success, R0: {:}", *ret_val_ptr);
      }
      else{
        println!("[+]Exec fail");
      }
    }
    
    ubpf_destroy_fn(vm);
    Ok(*ret_val_ptr)
  }
}

const DEBUG: bool = true;
const NBR_INSTR: usize = 3;

fn main() {
  let symbols = get_functions_pointer().unwrap();

  let start = std::time::Instant::now();
  const TOTAL_EXEC: i32 = 1_000_000;
  let mut i: i32 = 0;
  loop{
    i = i + 1;

    //-----------------------Replay crash---------------------------
    //let mut bytecode: [EbpfBytecode; NBR_INSTR + 1] = [
    //  EbpfBytecode::from(EbpfInstr::new(0xb7, 0x0, 0x6, 0x0, 0x0 as u32 as i32)),//r[6] = 0
    //  EbpfBytecode::from(EbpfInstr::new(0x17, 0x0, 0x6, 0x0, 0x1 as u32 as i32)),//[r6] = 0xFFFFFFFFFFF
    //  EbpfBytecode::from(EbpfInstr::new(0x7A, 0x0, 0x6, 0x0, 0x0 as u32 as i32)),//*(*(uint64_t *) (r[6] + off) = imm => 0xFFFFFFF + 0 = 0

    //  //Exit
    //  EbpfBytecode::from(EbpfInstr::new(0x95, 0x00, 0x00, 0x0000, 0x00000000 )),
    //];

    //for j in 0..NBR_INSTR as i32{
    //  if DEBUG{
    //    println!("{:03}=> {:?}", j, bytecode[j as usize]);
    //  }
    //}
    //-------------------------------------------------------------

    //-----------------------Find crash---------------------------
    let mut bytecode: [EbpfBytecode; NBR_INSTR + 1] = [EbpfBytecode::default(); NBR_INSTR + 1];

    for j in 0..NBR_INSTR as i32{
      let instr = EbpfInstr::generate_random_instr(j);
      let instr_byte = EbpfBytecode::from(instr);

      if DEBUG{
        println!("{:03}=> {:?} ---- {:?}", j, instr, instr_byte);
      }

      bytecode[j as usize] = instr_byte;
    }

    // instruction exit return r0
    bytecode[NBR_INSTR] = EbpfBytecode::from(EbpfInstr::new(0x95, 0x00, 0x00, 0x0000, 0x00000000));
    //-------------------------------------------------------------

    let ret = run_prgm(&bytecode, &symbols);

    if DEBUG{
      if let Ok(_k) = ret{
        println!("Ok")
      }
      if let Err(e) = ret {
        println!("Error {:?}", e);
      }
    }

    //if i % 10000 == 0 {
    //  let elapsed = start.elapsed().as_secs_f64();
    //  let avg_exec_time = i as f64 / elapsed;
    //  println!("=>{:}/{:} {:.2} exec/s", i, TOTAL_EXEC, avg_exec_time);
    //}
  }
}
