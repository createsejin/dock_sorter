use clap::Parser;
use std::collections::HashMap;

#[derive(Parser, Debug)]
#[command(author, version, about = "도크 라벨 출력 순서 및 범위 계산기", long_about = None)]
struct Args {
  /// 1차 출력 도크 목록 (공백으로 구분)
  #[arg(short = 'f', long, value_delimiter = ' ', num_args = 0.., required = false)]
  // 필수가 아님 (없을 수도 있음)
  first_priority: Vec<u32>,

  /// 2차 출력 도크 목록 (공백으로 구분)
  #[arg(short = 's', long, value_delimiter = ' ', num_args = 0.., required = false)]
  // 필수가 아님
  second_priority: Vec<u32>,

  /// 한 번에 출력할 도크 수
  #[arg(short = 'p', long)]
  per_page: usize,

  /// 처리할 최소 도크 번호 (필수)
  #[arg(long, required = true)]
  min: u32,

  /// 처리할 최대 도크 번호 (필수)
  #[arg(long, required = true)]
  max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Priority {
  First,  // 가장 높은 우선순위 (0)
  Second, // 다음 우선순위 (1)
  Third,  // 가장 낮은 우선순위 (2)
}

fn main() {
  let args = Args::parse();

  // 입력 유효성 검사
  if args.per_page == 0 {
    eprintln!("오류: 한 번에 출력할 도크 수는 1 이상이어야 합니다.");
    std::process::exit(1);
  }
  if args.min > args.max {
    eprintln!(
      "오류: 최소 도크 번호({})가 최대 도크 번호({})보다 클 수 없습니다.",
      args.min, args.max
    );
    std::process::exit(1);
  }

  // 1. 우선순위 맵 생성 및 범위 검사
  let mut priorities: HashMap<u32, Priority> = HashMap::new();
  let mut warnings: Vec<String> = Vec::new();

  for &dock in &args.first_priority {
    if dock >= args.min && dock <= args.max {
      priorities.insert(dock, Priority::First);
    } else {
      warnings.push(format!(
        "경고: 1차 도크 {}는 지정된 범위 [{}-{}] 밖에 있어 무시됩니다.",
        dock, args.min, args.max
      ));
    }
  }
  for &dock in &args.second_priority {
    if dock >= args.min && dock <= args.max {
      // 1차에 이미 있으면 덮어쓰지 않음
      priorities.entry(dock).or_insert(Priority::Second);
    } else {
      warnings.push(format!(
        "경고: 2차 도크 {}는 지정된 범위 [{}-{}] 밖에 있어 무시됩니다.",
        dock, args.min, args.max
      ));
    }
  }

  // 경고 출력
  for warning in warnings {
    eprintln!("{}", warning);
  }

  // 2. 전체 도크 목록 생성 (사용자 지정 범위 기준)
  let mut all_docks_sorted: Vec<u32> = Vec::new();
  for dock in args.min..=args.max {
    // 1차나 2차가 아니면 3차로 간주하고 맵에 추가 (존재하지 않는 경우)
    priorities.entry(dock).or_insert(Priority::Third);
    all_docks_sorted.push(dock);
  }
  // all_docks_sorted는 이미 min..=max 범위로 정렬되어 있음

  println!("처리 대상 도크 범위: {} - {}", args.min, args.max);
  println!("그룹당 출력 개수: {}", args.per_page);
  println!("--- 출력 순서 (1차: @, 2차: *) ---");

  // 3. 그룹핑 로직 (이전과 동일)
  let mut result_groups: Vec<Vec<u32>> = Vec::new();
  let mut current_group: Vec<u32> = Vec::new();

  for &dock in &all_docks_sorted {
    // min..=max 로 생성된 정렬된 전체 리스트 사용
    let current_priority = *priorities.get(&dock).unwrap_or(&Priority::Third); // 범위 내 모든 도크는 우선순위가 있음

    if current_group.is_empty() {
      current_group.push(dock);
    } else if current_group.len() >= args.per_page {
      result_groups.push(current_group);
      current_group = vec![dock];
    } else {
      let first_dock_in_group = current_group[0];
      let priority_of_first = *priorities.get(&first_dock_in_group).unwrap(); // 현재 그룹의 첫 도크는 반드시 맵에 있음

      if current_priority < priority_of_first {
        // 현재 도크 우선순위가 더 높으면 분리
        result_groups.push(current_group);
        current_group = vec![dock];
      } else {
        // 우선순위 같거나 낮으면 그룹에 추가
        current_group.push(dock);
      }
    }
  }

  // 마지막 그룹 추가
  if !current_group.is_empty() {
    result_groups.push(current_group);
  }

  // 4. 결과 출력 (기호 추가)
  for group in result_groups {
    let formatted_group: Vec<String> = group
      .iter()
      .map(|&d| {
        match priorities.get(&d) {
          Some(Priority::First) => format!("{}@", d),
          Some(Priority::Second) => format!("{}*", d), // 2차 기호 추가
          _ => d.to_string(),                          // 3차 또는 오류 시 (이론상 발생 안 함)
        }
      })
      .collect();
    println!("{}", formatted_group.join(", "));
  }
}
