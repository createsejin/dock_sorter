use clap::Parser;
use std::collections::{HashMap, HashSet};
// use std::cmp::Ordering; // Ordering은 명시적으로 사용되지 않으므로 제거해도 됩니다.

#[derive(Parser, Debug)]
#[command(author, version, about = "Dock Label Output Order and Range Calculator", long_about = None)]
struct Args {
  /// First priority docks. Can be single numbers or ranges (e.g., 1-3 5 7-9)
  #[arg(short = 'f', long, value_delimiter = ' ', num_args = 0.., required = false, value_parser = parse_dock_ranges, action = clap::ArgAction::Append)]
  first_priority: Vec<Vec<u32>>, // clap이 Vec<Vec<u32>>를 만들도록 하고, 나중에 flatten

  /// Second priority docks. Can be single numbers or ranges (e.g., 10-12 15)
  #[arg(short = 's', long, value_delimiter = ' ', num_args = 0.., required = false, value_parser = parse_dock_ranges, action = clap::ArgAction::Append)]
  second_priority: Vec<Vec<u32>>, // clap이 Vec<Vec<u32>>를 만들도록 하고, 나중에 flatten

  /// Exception docks to be grouped together, ignoring -p. (e.g., 1-3 7-9 10)
  #[arg(long = "exceptions", short = 'e', value_delimiter = ' ', num_args = 0.., required = false, value_parser = parse_dock_ranges_for_exceptions, action = clap::ArgAction::Append)]
  exception_groups_raw: Vec<Vec<u32>>, // 각 예외 그룹을 Vec<u32>로 받음

  /// Number of docks to print per group
  #[arg(short = 'p', long)]
  per_page: usize,

  /// Minimum dock number to process
  #[arg(long, required = false, default_value_t = 51)] // 기본값 51로 설정, 필수가 아님
  min: u32,

  /// Maximum dock number to process (required)
  #[arg(long, required = true)]
  max: u32,
}

/// 입력된 문자열(단일 숫자 또는 "숫자-숫자" 범위)을 파싱하여 u32의 Vec으로 변환하는 함수.
/// clap의 value_parser로 사용됩니다.
fn parse_dock_ranges(s: &str) -> Result<Vec<u32>, String> {
  let mut docks = Vec::new();
  if s.contains('-') {
    let parts: Vec<&str> = s.splitn(2, '-').collect();
    if parts.len() == 2 {
      let start_str = parts[0].trim();
      let end_str = parts[1].trim();
      if let (Ok(start), Ok(end)) = (start_str.parse::<u32>(), end_str.parse::<u32>()) {
        if start <= end {
          for i in start..=end {
            docks.push(i);
          }
        } else {
          return Err(format!(
            "Invalid range: start ({}) must be less than or equal to end ({}) in '{}'",
            start, end, s
          ));
        }
      } else {
        return Err(format!(
          "Invalid range format: '{}'. Both parts must be numbers.",
          s
        ));
      }
    } else {
      // 이 경우는 splitn(2, ..) 로 인해 발생하지 않지만, 완전성을 위해
      return Err(format!("Invalid range format: '{}'", s));
    }
  } else if let Ok(dock_num) = s.trim().parse::<u32>() {
    // collapsible_else_if 수정: else if let 사용
    docks.push(dock_num);
  } else {
    // if let이 실패한 경우 (즉, '-'도 없고, 단일 숫자 파싱도 실패한 경우)
    return Err(format!("Invalid number or range format: '{}'", s));
  }
  Ok(docks)
}

/// 예외 그룹용 파서. parse_dock_ranges와 동일하지만, clap 어트리뷰트에서 명시적으로 구분하기 위함.
/// 실제로는 parse_dock_ranges를 그대로 사용해도 무방하나, 의미상 분리.
fn parse_dock_ranges_for_exceptions(s: &str) -> Result<Vec<u32>, String> {
  parse_dock_ranges(s)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Priority {
  Exception, // 0: 예외 그룹 (가장 높음)
  First,     // 1: 1차
  Second,    // 2: 2차
  Third,     // 3: 3차 (일반)
}

fn main() {
  let args_raw = Args::parse(); // clap으로부터 직접 파싱된 값

  // Vec<Vec<u32>> 를 Vec<u32> 로 flatten 하고, 중복 제거 및 정렬 (선택 사항이지만 권장)
  let first_priority_flat: Vec<u32> = args_raw.first_priority.into_iter().flatten().collect();
  let second_priority_flat: Vec<u32> = args_raw.second_priority.into_iter().flatten().collect();
  // exception_groups_raw는 Vec<Vec<u32>> 형태 그대로 유지. 각 내부 Vec<u32>가 하나의 예외 그룹임.
  let exception_groups: Vec<Vec<u32>> = args_raw.exception_groups_raw;

  // 이제 args_raw 대신 flatten된 Vec<u32>를 사용합니다.
  // 더 간결하게 하려면, Args 구조체를 수정하는 대신 여기서 바로 변환된 값을 사용하는 것도 방법입니다.
  // 여기서는 명확성을 위해 별도의 변수를 사용합니다.

  // 입력 유효성 검사
  if args_raw.per_page == 0 {
    // per_page는 args_raw에서 직접 접근
    eprintln!("Error: Number of docks per group must be 1 or greater.");
    std::process::exit(1);
  }
  if args_raw.min > args_raw.max {
    // min, max도 args_raw에서 직접 접근
    eprintln!(
      "Error: Minimum dock number ({}) cannot be greater than maximum dock number ({}).",
      args_raw.min, args_raw.max
    );
    std::process::exit(1);
  }

  // 1. 우선순위 맵 생성 및 범위 검사
  let mut priorities: HashMap<u32, Priority> = HashMap::new();
  let mut warnings: Vec<String> = Vec::new();
  let mut exception_docks_set: HashSet<u32> = HashSet::new(); // 예외 그룹에 속한 모든 도크 추적

  // 예외 그룹 우선순위 할당
  for group in &exception_groups {
    for &dock in group {
      if dock >= args_raw.min && dock <= args_raw.max {
        priorities.insert(dock, Priority::Exception);
        exception_docks_set.insert(dock);
      } else {
        warnings.push(format!(
        "Warning: Exception dock {} from group {:?} is outside the specified range [{}-{}] and will be ignored.",
        dock, group, args_raw.min, args_raw.max
        ));
      }
    }
  }

  // 1차 우선순위 할당 (예외가 아닌 경우에만)
  for &dock in &first_priority_flat {
    if dock >= args_raw.min && dock <= args_raw.max && !exception_docks_set.contains(&dock) {
      priorities.insert(dock, Priority::First);
    } else if !(dock >= args_raw.min && dock <= args_raw.max) {
      // 범위 밖 경고
      warnings.push(format!(
        "Warning: First priority dock {} is outside the specified range [{}-{}] and will be ignored.",
        dock, args_raw.min, args_raw.max
      ));
    }
    // 예외 그룹에 이미 속해있으면 우선순위 변경 안 함
  }

  // 2차 우선순위 할당 (예외나 1차가 아닌 경우에만)
  for &dock in &second_priority_flat {
    if dock >= args_raw.min && dock <= args_raw.max && !exception_docks_set.contains(&dock) {
      priorities.entry(dock).or_insert(Priority::Second); // First가 있으면 덮어쓰지 않음
    } else if !(dock >= args_raw.min && dock <= args_raw.max) {
      // 범위 밖 경고
      warnings.push(format!(
        "Warning: Second priority dock {} is outside the specified range [{}-{}] and will be ignored.",
        dock, args_raw.min, args_raw.max
      ));
    }
  }

  // 경고 출력
  for warning in warnings {
    eprintln!("{}", warning);
  }

  // 2. 전체 도크 목록 생성 (사용자 지정 범위 기준) 및 3차 우선순위 부여
  let mut all_docks_to_process: Vec<u32> = Vec::new();
  for dock in args_raw.min..=args_raw.max {
    priorities.entry(dock).or_insert(Priority::Third); // Exception, First, Second가 아니면 Third
    all_docks_to_process.push(dock); // 정렬된 순서대로 추가됨
  }

  println!("Processing dock range: {} - {}", args_raw.min, args_raw.max);
  println!("Docks per group (non-exceptions): {}", args_raw.per_page);
  if !exception_groups.is_empty() {
    println!("Exception groups (printed together):");
    for group in &exception_groups {
      let group_str: Vec<String> = group
        .iter()
        .filter(|&&d| d >= args_raw.min && d <= args_raw.max)
        .map(|d| d.to_string())
        .collect();
      if !group_str.is_empty() {
        println!("  - [{}]", group_str.join(", "));
      }
    }
  }
  println!("--- Output Order (Exc: group, 1st: @, 2nd: *) ---");

  // 3. 그룹핑 로직 수정
  let mut result_groups: Vec<Vec<u32>> = Vec::new();
  let mut processed_docks: HashSet<u32> = HashSet::new(); // 이미 처리된 도크 (주로 예외 그룹)

  // 3.1 예외 그룹 먼저 처리
  // 예외 그룹은 입력된 순서대로, 그리고 그룹 내 도크도 입력된 순서대로 처리
  for ex_group in &exception_groups {
    let current_ex_group: Vec<u32> = ex_group
      .iter()
      .cloned()
      .filter(|&d| d >= args_raw.min && d <= args_raw.max && !processed_docks.contains(&d)) // 범위 내, 미처리 도크만
      .collect();

    if !current_ex_group.is_empty() {
      result_groups.push(current_ex_group.clone());
      for &dock in &current_ex_group {
        processed_docks.insert(dock);
      }
    }
  }

  // 3.2 나머지 도크 처리 (all_docks_to_process는 min부터 max까지 정렬된 상태)
  let mut current_regular_group: Vec<u32> = Vec::new();
  for &dock in &all_docks_to_process {
    if processed_docks.contains(&dock) {
      // 이미 예외 그룹으로 처리된 도크는 건너뜀
      continue;
    }

    let current_priority = *priorities.get(&dock).unwrap(); // 이 시점엔 모든 도크에 우선순위 할당됨

    // 현재 도크가 Exception이면 안됨 (이미 위에서 처리) - 안전장치
    if current_priority == Priority::Exception {
      // 혹시 누락된 예외 도크가 있다면 개별 처리 (이론상 발생 안해야 함)
      if !current_regular_group.is_empty() {
        result_groups.push(current_regular_group);
        current_regular_group = Vec::new();
      }
      result_groups.push(vec![dock]); // 개별 예외 도크 그룹
      processed_docks.insert(dock); // 처리됨 표시
      continue;
    }

    if current_regular_group.is_empty() {
      current_regular_group.push(dock);
    } else if current_regular_group.len() >= args_raw.per_page {
      result_groups.push(current_regular_group);
      current_regular_group = vec![dock];
    } else {
      let first_dock_in_group = current_regular_group[0];
      let priority_of_first = *priorities.get(&first_dock_in_group).unwrap();

      if current_priority < priority_of_first {
        result_groups.push(current_regular_group);
        current_regular_group = vec![dock];
      } else {
        current_regular_group.push(dock);
      }
    }
  }

  if !current_regular_group.is_empty() {
    result_groups.push(current_regular_group);
  }

  // 4. 결과 출력
  for group in result_groups {
    let formatted_group: Vec<String> = group
      .iter()
      .map(|&d| {
        match priorities.get(&d) {
          Some(Priority::Exception) => format!("{}", d), // 예외 표시 (E 또는 다른 기호)
          Some(Priority::First) => format!("{}@", d),
          Some(Priority::Second) => format!("{}*", d),
          _ => d.to_string(),
        }
      })
      .collect();
    println!("{}", formatted_group.join(", "));
  }
}
