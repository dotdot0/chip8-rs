const RAM_SIZE: usize = 4096;
const REG_SIZE: usize = 16;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const STACK_SIZE: usize = 16;
const START_ADDR: u16 = 0x200;
const NUM_KEYS: usize = 16;
const FONTSET_SIZE: usize = 80;

const FONTSET: [u8; FONTSET_SIZE] = [
0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
0x20, 0x60, 0x20, 0x20, 0x70, // 1
0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
0x90, 0x90, 0xF0, 0x10, 0x10, // 4
0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
0xF0, 0x10, 0x20, 0x40, 0x40, // 7
0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
0xF0, 0x90, 0xF0, 0x90, 0x90, // A
0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
0xF0, 0x80, 0x80, 0x80, 0xF0, // C
0xE0, 0x90, 0x90, 0x90, 0xE0, // D
0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
0xF0, 0x80, 0xF0, 0x80, 0x80 // F
];

pub struct Cpu{
  memory: [u8; RAM_SIZE],
  pc: u16,
  reg: [u8; REG_SIZE],
  display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
  keys: [bool; NUM_KEYS],
  i_reg: u16,
  sp: u16,
  stack: [u16; STACK_SIZE],
  dt: u8,
  st: u8
}

impl Cpu{
  pub fn new() -> Self{
    let mut new_cpu = Self{ 
      memory: [0; RAM_SIZE],
      pc: START_ADDR,
      reg: [0; REG_SIZE],
      display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
      i_reg : 0,
      sp : 0,
      stack : [0; STACK_SIZE],
      keys: [false; NUM_KEYS],
      dt: 0,
      st: 0
    };

    new_cpu.memory[..FONTSET_SIZE].copy_from_slice(&FONTSET);

    new_cpu
  }

  fn push(&mut self, val: u16){
    if self.sp < STACK_SIZE as u16{
      self.stack[self.sp as usize] = val;
      self.sp+=1;
    }else{
      println!("Stack Overflow!");
    }
  }    

  fn pop(&mut self) -> u16{
    if self.sp > 0{
      self.sp -= 1;
      return self.stack[self.sp as usize];
    }
    return 2200;
  }

  pub fn run(&mut self){
    let op_code = self.fetch();
    self.execute(op_code);
  }

  fn fetch(&mut self) -> u16{
    let lower_byte = self.memory[self.pc as usize] as u16;
    let upper_byte = self.memory[(self.pc+1) as usize] as u16;
    let op_code = (upper_byte << 8) | lower_byte;
    self.pc+=2;
    op_code
  }

  fn execute(&mut self, opcode: u16){
    let digit1 = (opcode & 0xF000) >> 12;
    let digit2 = (opcode & 0x0F00) >> 8;
    let digit3 = (opcode & 0x00F0) >> 4;
    let digit4 = opcode & 0x000F;

    match (digit1, digit2, digit3, digit4) {
       (0, 0, 0, 0) => return,

       (0, 0, 0xE, 0) => {
        self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
       },

       (0, 0, 0xE, 0xE) => {
        let ret_addr = self.pop();
        self.pc = ret_addr;
       },

       (1, _, _, _) => {
        let addr = opcode & 0xfff;
        self.pc = addr;
       }

       (2, _, _, _) => {
        self.push(self.pc);
        let addr = opcode & 0xfff;
        self.pc = addr;
       }

       (3, _, _, _) => {
        let x = digit2 as usize;
        let val = (opcode & 0xff) as u8;
        if self.reg[x] == val {
          self.pc+=2;
        }
       },

       (4, _, _, _) => {
        let x = digit2 as usize;
        let val = (opcode & 0xff) as u8;
        if self.reg[x]!=val {
          self.pc+=2;
        }
       }, 

       (5, _, _, 0) => {
        let x = digit2 as usize;
        let y = digit3 as usize;
        if self.reg[x] == self.reg[y]{
          self.pc+=2;
        }
       }

       (6, _, _, _) => {
        let x = digit2 as usize;
        let val = (opcode & 0xff) as u8;
        self.reg[x] = val;
       }

       (7, _, _, _) => {
        let x = digit2 as usize;
        let val = (opcode & 0xff) as u8;
        self.reg[x]+=val;
       }

       (8, _, _, 0) => {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.reg[x] = self.reg[y];
       }

       (8, _, _, 1) => {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.reg[x] |= self.reg[y];
       }, 

       (8, _, _, 2) => {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.reg[x] &= self.reg[y];
       },

       (8, _, _, 3) => {
        let x = digit2 as usize;
        let y = digit3 as usize;
        self.reg[x] ^= self.reg[y];
       },

       (8, _, _, 4) => {
        let x = digit2 as usize;
        let y = digit3 as usize;
        let (reg_x, carry) = self.reg[x].overflowing_add(self.reg[y]);
        self.reg[0xf] = if carry {1} else {0}; 
        self.reg[x] = reg_x;
       },

       (_, _, _, _) => todo!()
    }
  }
}