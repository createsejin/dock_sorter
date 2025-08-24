#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority { // 우선순위
  First,     // 1: 1차
  Second,    // 2: 2차
  Third,     // 3: 3차 (일반)
}

