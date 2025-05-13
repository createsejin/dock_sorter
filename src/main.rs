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
  First,     // 1: 1차
  Second,    // 2: 2차
  Third,     // 3: 3차 (일반)
}

fn main() {
  let args_raw = Args::parse();

  // 입력 유효성 검사
  if args_raw.per_page == 0 {
    eprintln!("Error: Number of docks per group must be 1 or greater.");
    std::process::exit(1);
  }
  if args_raw.min > args_raw.max {
    eprintln!(
      "Error: Minimum dock number ({}) cannot be greater than maximum dock number ({}).",
      args_raw.min, args_raw.max
    );
    std::process::exit(1);
  }

  // 1. 입력된 우선순위 및 예외 도크 정리
  let first_priority_docks: HashSet<u32> = args_raw.first_priority.into_iter().flatten().collect();
  let second_priority_docks: HashSet<u32> = args_raw.second_priority.into_iter().flatten().collect();
  
  // 예외 그룹 처리: 각 예외 그룹을 정렬하고, 전체 예외 도크 집합을 만듦.
  // final_exception_groups는 각 예외 그룹(Vec<u32>)의 리스트. 각 그룹은 정렬됨.
  // all_exception_docks는 모든 예외 도크를 담는 HashSet.
  let mut final_exception_groups: Vec<Vec<u32>> = Vec::new();
  let mut all_exception_docks: HashSet<u32> = HashSet::new();
  let mut warnings: Vec<String> = Vec::new();

  for raw_ex_group in args_raw.exception_groups_raw { // Vec<Vec<u32>>
    let mut current_ex_group: Vec<u32> = raw_ex_group.into_iter()
      .filter(|&d| { // 범위 검사
        if d >= args_raw.min && d <= args_raw.max {
          true
        } else {
          warnings.push(format!(
            "Warning: Exception dock {} is outside the specified range [{}-{}] and will be ignored from its group.",
            d, args_raw.min, args_raw.max
          ));
          false
        }
      })
      .collect();
    
    current_ex_group.sort_unstable(); // 그룹 내 도크 정렬
    current_ex_group.dedup();         // 그룹 내 중복 제거

    if !current_ex_group.is_empty() {
      // 다른 예외 그룹과 겹치는 도크가 있는지 확인하고, 겹치면 경고 후 현재 그룹에서 제외
      // 또는 다른 전략 (예: 먼저 정의된 그룹 우선) - 여기서는 단순화를 위해 겹치면 경고만
      let mut filtered_current_ex_group = Vec::new();
      for dock in current_ex_group {
          if all_exception_docks.contains(&dock) {
              warnings.push(format!(
                  "Warning: Dock {} is part of multiple exception groups. It will remain in the first encountered group.",
                  dock
              ));
          } else {
              filtered_current_ex_group.push(dock);
              all_exception_docks.insert(dock);
          }
      }
      if !filtered_current_ex_group.is_empty() {
        final_exception_groups.push(filtered_current_ex_group);
      }
    }
  }
  // final_exception_groups를 첫 번째 도크 번호 기준으로 정렬 (출력 순서 일관성 위함)
  final_exception_groups.sort_unstable_by_key(|group| group.first().cloned().unwrap_or(u32::MAX));


  // 2. 각 도크에 우선순위 할당 (예외 도크 제외)
  let mut priorities: HashMap<u32, Priority> = HashMap::new();

  for &dock in &first_priority_docks {
    if dock >= args_raw.min && dock <= args_raw.max && !all_exception_docks.contains(&dock) {
      priorities.insert(dock, Priority::First);
    } else if !(dock >= args_raw.min && dock <= args_raw.max) { // 범위 밖 경고
      warnings.push(format!(
        "Warning: First priority dock {} is outside the specified range [{}-{}] and will be ignored.",
        dock, args_raw.min, args_raw.max
      ));
    }
  }

  for &dock in &second_priority_docks {
    if dock >= args_raw.min && dock <= args_raw.max && !all_exception_docks.contains(&dock) {
      priorities.entry(dock).or_insert(Priority::Second);
    } else if !(dock >= args_raw.min && dock <= args_raw.max) { // 범위 밖 경고
       warnings.push(format!(
        "Warning: Second priority dock {} is outside the specified range [{}-{}] and will be ignored.",
        dock, args_raw.min, args_raw.max
      ));
    }
  }
  // 3차 우선순위는 나중에 그룹핑 시점에 기본값으로 처리

  // 경고 메시지 출력
  for warning in warnings {
    eprintln!("{}", warning);
  }

  // 3. 처리할 전체 도크 목록 (min부터 max까지 정렬됨)
  let all_docks_in_range: Vec<u32> = (args_raw.min..=args_raw.max).collect();

  println!("Processing dock range: {} - {}", args_raw.min, args_raw.max);
  println!("Docks per group (non-exceptions): {}", args_raw.per_page);
  if !final_exception_groups.is_empty() {
    println!("Exception groups (will be printed together, in order of their first dock):");
    for ex_group in &final_exception_groups {
        let group_str: Vec<String> = ex_group.iter().map(|d| d.to_string()).collect();
        println!("  - [{}]", group_str.join(", "));
    }
  }
  println!("--- Output Order (1st: @, 2nd: *) ---");

  // 4. 최종 그룹핑 로직
  let mut result_groups: Vec<Vec<u32>> = Vec::new();
  let mut processed_docks_in_grouping: HashSet<u32> = HashSet::new(); // 그룹핑 과정에서 처리된 도크

  for &current_dock in &all_docks_in_range {
    if processed_docks_in_grouping.contains(&current_dock) {
      continue; // 이미 이전 그룹에 포함되어 처리됨
    }

    // 현재 도크가 예외 그룹의 시작점인지 확인
    let mut is_exception_start = false;
    let mut current_exception_group_data: Option<Vec<u32>> = None;

    if all_exception_docks.contains(&current_dock) {
      for ex_g in &final_exception_groups {
        if ex_g.first() == Some(&current_dock) { // 현재 도크가 이 예외 그룹의 시작점
          is_exception_start = true;
          current_exception_group_data = Some(ex_g.clone());
          break;
        }
      }
    }

    if is_exception_start && current_exception_group_data.is_some() {
      // 예외 그룹 처리
      let ex_group = current_exception_group_data.unwrap();
      result_groups.push(ex_group.clone());
      for &dock_in_ex in &ex_group {
        processed_docks_in_grouping.insert(dock_in_ex);
      }
    } else if !all_exception_docks.contains(&current_dock) { // 예외 그룹의 일부가 아닌 일반 도크 처리
      let mut regular_group: Vec<u32> = Vec::new();
      regular_group.push(current_dock);
      processed_docks_in_grouping.insert(current_dock);

      // 다음 도크들을 보면서 그룹 확장
      let mut next_dock_idx_in_range = all_docks_in_range.iter().position(|&d| d == current_dock).unwrap_or(0) + 1;
      
      while regular_group.len() < args_raw.per_page && next_dock_idx_in_range < all_docks_in_range.len() {
        let next_dock_candidate = all_docks_in_range[next_dock_idx_in_range];

        if processed_docks_in_grouping.contains(&next_dock_candidate) || all_exception_docks.contains(&next_dock_candidate) {
          break; // 이미 처리되었거나, 다음이 예외 그룹에 속하면 현재 일반 그룹 종료
        }

        let first_in_regular_prio = priorities.get(&regular_group[0]).unwrap_or(&Priority::Third);
        let next_candidate_prio = priorities.get(&next_dock_candidate).unwrap_or(&Priority::Third);

        if next_candidate_prio < first_in_regular_prio { // 다음 도크 우선순위가 더 높으면 그룹 분리
          break;
        }

        regular_group.push(next_dock_candidate);
        processed_docks_in_grouping.insert(next_dock_candidate);
        next_dock_idx_in_range += 1;
      }
      result_groups.push(regular_group);
    }
    // 만약 현재 도크가 예외 그룹의 시작점이 아니고, 예외 그룹의 중간이나 끝에 속한 도크라면,
    // processed_docks_in_grouping 체크에 의해 이 루프의 다음 반복에서 continue로 건너뛰어짐.
  }

  // 5. 결과 출력
  for group in result_groups {
    let formatted_group: Vec<String> = group
      .iter()
      .map(|&d| {
        if all_exception_docks.contains(&d) { // 예외 도크는 기호 없음
          d.to_string()
        } else {
          match priorities.get(&d) { // 예외가 아닌 도크는 priorities 맵에 우선순위가 있어야 함
            Some(Priority::First) => format!("{}@", d),
            Some(Priority::Second) => format!("{}*", d),
            Some(Priority::Third) | None => d.to_string(), // 3차 또는 혹시 모를 누락(None)
          }
        }
      })
      .collect();
    println!("{}", formatted_group.join(", "));
  }
}
