use clap::Parser;
use std::collections::{HashMap, HashSet};
// use std::cmp::Ordering; // Ordering은 명시적으로 사용되지 않으므로 제거해도 됩니다.

#[derive(Parser, Debug)]
#[command(author, version, about = "Dock Label Output Order and Range Calculator", long_about = None)]
struct Args {
  /// First priority docks. Can be single numbers or ranges (e.g., 1-3 5 7-9)
  #[arg(short = 'f', long, value_delimiter = ' ', num_args = 0.., required = false, value_parser = parse_dock_ranges, action = clap::ArgAction::Append)]
  first_priority: Vec<Vec<u32>>, // clap이 Vec<Vec<u32>>를 만들도록 하고, 나중에 flatten
  // 예를들어서 -f 65-66 71 56 62 이런식으로 입력됐다면,
  // parse_dock_ranges 함수에 의해 각각 [[65, 66], [71], [56], [62]] 이런식으로 리스트가 만들어진다.

  /// Second priority docks. Can be single numbers or ranges (e.g., 10-12 15)
  #[arg(short = 's', long, value_delimiter = ' ', num_args = 0.., required = false, value_parser = parse_dock_ranges, action = clap::ArgAction::Append)]
  second_priority: Vec<Vec<u32>>, // clap이 Vec<Vec<u32>>를 만들도록 하고, 나중에 flatten

  /// Exception docks to be grouped together, ignoring -p. (e.g., 1-3 7-9 10)
  #[arg(long = "exceptions", short = 'e', value_delimiter = ' ', num_args = 0.., required = false, value_parser = parse_dock_ranges, action = clap::ArgAction::Append)]
  exception_groups_raw: Vec<Vec<u32>>, // 각 예외 그룹을 Vec<u32>로 받음
  // 예외 그룹은 1-3 같은 연속 범위나 10 같은 단일 그룹으로 지정될 수 있다.
  // _raw는 flatten되지 않은 [[1, 2, 3], [10]] 같은 형식의 Vec이다.

  /// Number of docks to print per group
  #[arg(short = 'p', long)]
  per_page: usize,

  /// Number of docks per group for 1st priority docks (defaults to -p value if not set)
  #[arg(short = '1', long = "fp", required = false)] // short: -1, long: --fpp
  first_priority_per_page: Option<usize>,

  /// Number of docks per group for 2nd priority docks (defaults to -p value if not set)
  #[arg(short = '2', long = "sp", required = false)] // short: -2, long: --spp
  second_priority_per_page: Option<usize>,

  /// Minimum dock number to process
  #[arg(long, required = false, default_value_t = 51)] // 기본값 51로 설정, optional
  min: u32,

  /// Maximum dock number to process 
  #[arg(long, required = false, default_value_t = 78)] // 기본값 78로 설정, optional
  max: u32,
}

/// 입력된 문자열(단일 숫자 또는 "숫자-숫자" 범위)을 파싱하여 u32의 Vec으로 변환하는 함수.
/// clap의 value_parser로 사용됩니다.
fn parse_dock_ranges(s: &str) -> Result<Vec<u32>, String> {
  // 파싱된 도크 숫자들이 저장될 Vec
  let mut docks = Vec::new();

  if s.contains('-') { // 만약 arg가 `-`를 포함한다면
    // 두 개의 숫자로 split 한다.
    let parts: Vec<&str> = s.splitn(2, '-').collect();
    if parts.len() == 2 { // split된 parts가 2개라면
      let start_str = parts[0].trim(); // parts[0]을 trim하여 start_str에 저장한다.
      let end_str = parts[1].trim(); // 마찬가지로 parts[1]을 trim하여 end_str에 저장한다.
      // 만약 start_str와 end_str를 u32로 파싱하는게 Ok라면 파싱된 값을 start와 end에 할당한다.
      if let (Ok(start), Ok(end)) = (start_str.parse::<u32>(), end_str.parse::<u32>()) {
        if start <= end { // start가 end보다 작거나 같다면
          for i in start..=end { // start에서 시작하여 end를 포함하여 범위를 생성하고
            docks.push(i); // 범위에서 생성된 숫자 i를 docks에 push한다.
          }
        } else { // start가 end보다 큰 경우
          return Err(format!(
            // 에러 메세지를 내뱉는다.
            "Invalid range: start ({start}) must be less than or equal to end ({end}) in '{s}'"
          ));
        }
      } else { // u32 파싱에 실패한경우. 입력된 문자열이 숫자 형식이 아니라서 발생할 수 있음.
        return Err(format!(
          "Invalid range format: '{s}'. Both parts must be numbers."));
      }
    } else {
      // 이 경우는 splitn(2, ..) 로 인해 발생하지 않지만, 완전성을 위해
      return Err(format!("Invalid range format: '{s}'"));
    }
  } 
  // 만약 `-`가 포함되지 않은 일반 숫자라서 arg s를 trim한뒤 parsing에 성공했다면
  // 파싱된 수를 dock_num에 할당하고
  else if let Ok(dock_num) = s.trim().parse::<u32>() {
    // docks에 push 한다.
    docks.push(dock_num);
  } else {
    // 그외의 경우. 즉, '-'도 없고, 단일 숫자 파싱도 실패한 경우
    return Err(format!("Invalid number or range format: '{s}'"));
  }
  // 에러가 없다면 docks를 Result로 return 한다.
  Ok(docks)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Priority { // 우선순위
  First,     // 1: 1차
  Second,    // 2: 2차
  Third,     // 3: 3차 (일반)
}

fn main() {
  // 입력된 arg들을 얻는다.
  let args_raw = Args::parse();

  // 입력 유효성 검사
  // per_page들이 0인지 검사한다.
  if args_raw.per_page == 0 || 
     (args_raw.first_priority_per_page == Some(0)) ||
     (args_raw.second_priority_per_page == Some(0)) {
    eprintln!("Error: Number of docks per group must be 1 or greater for all per-page settings.");
    std::process::exit(1);
  }
  // min과 max를 비교하여 min이 max보다 큰 경우 에러 출력 후 프로그램 종료
  if args_raw.min > args_raw.max {
    eprintln!(
      "Error: Minimum dock number ({}) cannot be greater than maximum dock number ({}).",
      args_raw.min, args_raw.max
    );
    std::process::exit(1);
  }

  // per_page 값 결정 로직
  // first와 second는 optional한 값이므로 값이 없다면 per_page를 따르도록 한다.
  let fpp = args_raw.first_priority_per_page.unwrap_or(args_raw.per_page);
  let spp = args_raw.second_priority_per_page.unwrap_or(args_raw.per_page);
  let gpp = args_raw.per_page; // general per page(third)

  // 1. 입력된 우선순위 및 예외 도크 정리
  // -f 65-66 71 56 62 와 같이 입력했다면 [[65, 66], [71], [56], [62]] 이런식인데, 이걸 flatten을 이용해서
  // [65, 66, 71, 56, 62] 이렇게 만들어 HashSet에 저장해준다.
  let first_priority_docks: HashSet<u32> = args_raw.first_priority.into_iter().flatten().collect();
  let second_priority_docks: HashSet<u32> = args_raw.second_priority.into_iter().flatten().collect();
  
  // 예외 그룹 처리: 각 예외 그룹을 정렬하고, 전체 예외 도크 집합을 만듦.
  // 최종적인 exception_group Vec들이 들어갈 Vec이다.
  let mut final_exception_groups: Vec<Vec<u32>> = Vec::new();
  // args_raw.exception_groups_raw에서의 모든 예외 도크들을 담는 HashSet.
  let mut all_exception_docks: HashSet<u32> = HashSet::new();
  // 범위 밖을 벗어난 입력값이 있다면 해당 값을 경고 메세지에 지정한 뒤 경고 메세지들을 저장하여 나중에 출력하기 위한 Vec다.
  let mut warnings: Vec<String> = Vec::new();

  // args에서 exception_groups_raw에 접근하여 각 raw_ex_group Vec을 순회한다.
  for raw_ex_group in args_raw.exception_groups_raw {
    // raw_ex_group에서 각 숫자들을 검사하여 min과 max 사이의 값인지를 필터링하여 current_ex_group을 얻는다.
    let mut current_ex_group: Vec<u32> = raw_ex_group.into_iter()
      .filter(|&d| {
        // raw_ex_group의 각 숫자가 min과 max 사이의 값인지를 필터링한다.
        if d >= args_raw.min && d <= args_raw.max { true } 
        else { // min max 값 이외의 범위에 있는 숫자라면 ignored되고 해당 숫자는 경고 메세지에 저장되어 
          // 이 메세지를 warnings에 담아둔다.
          warnings.push(
            format!("Warning: Exception dock {} is outside the specified range [{}-{}] and will be ignored.", 
              d, args_raw.min, args_raw.max));
          false // 이 경우에는 false로 처리하여 필터링한다.
        }
      }).collect();
    // current_ex_group을 sort한다.
    current_ex_group.sort_unstable();
    // current_ex_group에서 중복 항목을 제거한다.
    current_ex_group.dedup();
    // 만약 current_ex_group이 비어있지 않다면
    if !current_ex_group.is_empty() {
      let mut filtered_group = Vec::new();
      // current_ex_group을 순회한다.
      for dock in current_ex_group {
        // 만약 all_exception_docks가 현재 순회 dock를 포함하고 있지 않다면
        if !all_exception_docks.contains(&dock) {
          // filtered_group에 push하고,
          filtered_group.push(dock);
          // all_exception_docks에 insert한다.
          all_exception_docks.insert(dock);
        } else { // 만약 all_exception_docks가 현재 dock를 포함한다면(중복)
          // warnings에 push하고 해당 dock의 경고 메세지를 warnings Vec에 저장해둔다.
          warnings.push(format!("Warning: Dock {dock} in exception group already part of another exception group. Ignoring."));
        }
      }
      // 현재의 crrent_ex_group의 순회가 종료된 후 filtered_group이 무언가 있다면
      if !filtered_group.is_empty() {
        // final_exception_groups에 filtered_group을 insert한다.
        final_exception_groups.push(filtered_group);
      }
    }
  }
  // final_exception_groups을 sort하는데, 각 그룹들의 첫머리 숫자 기준으로 sort한다.
  // group의 .first로 첫 숫자를 추출하고, cloned로 복사한뒤 unwrap_or로 해당 숫자를 얻거나 u32의 MAX값을 추출한다.
  // 추출한 값을 기준으로 final_exception_groups를 sort한다. 
  final_exception_groups.sort_unstable_by_key(|group| group.first().cloned().unwrap_or(u32::MAX));

  // 2. 각 도크에 우선순위 할당 (예외 도크 제외)
  // 도크 숫자를 key로, Priority를 value로 갖는 HashMap을 생성한다. 
  let mut priorities: HashMap<u32, Priority> = HashMap::new();

  // 1차 그룹의 dock들을 순회한다.
  for &dock in &first_priority_docks {
    // 각 dock가 min보다 크거나 같고, max보다 작거나 같고, all_exception_docks에 포함되지 않았다면
    if dock >= args_raw.min && dock <= args_raw.max && !all_exception_docks.contains(&dock) {
      // 해당 dock를 priorites HashMap에 dock를 key로, Priority::First를 value로 insert한다.
      priorities.insert(dock, Priority::First);
    } // 그게 아니라 min max 범위를 벗어난 값이 있다면
    else if !(dock >= args_raw.min && dock <= args_raw.max) { // 범위 밖 경고
      // warnings에 해당 dock의 경고 메세지를 저장한다.
      warnings.push(format!(
        "Warning: First priority dock {} is outside the specified range [{}-{}] and will be ignored.",
        dock, args_raw.min, args_raw.max
      ));
    }
  }

  // 2차 그룹도 1차 그룹과 같은 방식으로 처리한다.
  for &dock in &second_priority_docks {
    if dock >= args_raw.min && dock <= args_raw.max && !all_exception_docks.contains(&dock) {
      // 이 경우에는 Priority::Second를 값으로 넣어둔다.
      priorities.entry(dock).or_insert(Priority::Second);
    } else if !(dock >= args_raw.min && dock <= args_raw.max) { // 범위 밖 경고
       warnings.push(format!(
        "Warning: Second priority dock {} is outside the specified range [{}-{}] and will be ignored.",
        dock, args_raw.min, args_raw.max
      ));
    }
  }
  // 3차 우선순위는 나중에 그룹핑 시점에 기본값으로 처리한다.

  // 경고 메시지를 출력한다.
  for warning in warnings {
    eprintln!("{warning}");
  }

  // 처리할 전체 도크 목록 = min부터 max까지의 처리할 모든 도크가 담긴 Vec이다.
  let all_docks_in_range: Vec<u32> = (args_raw.min..=args_raw.max).collect();

  // 처리 도크의 min..max 도크 range를 출력한다.
  println!("Processing dock range: {} - {}", args_raw.min, args_raw.max);
  // 1차, 2차 그룹, 일반 그룹의 각 처리당 per-page들을 출력한다.
  println!("Docks per group (1st priority): {fpp}");
  println!("Docks per group (2nd priority): {spp}");
  println!("Docks per group (3rd priority/general): {gpp}");

  // 만약 final_exception_groups이 있는 경우 해당 그룹들을 출력해준다.
  if !final_exception_groups.is_empty() {
    println!("Exception groups (printed together, in order of their first dock):");
    // final_exception_groups의 각 그룹들을 순회한다.
    for ex_group in &final_exception_groups {
      // 각 ex_group을 iter().map하여 각 dock인 d를 string으로 만든뒤 이것을 다시 Vec으로 collect한뒤 이 Vec을 
      // join을 이용하여 하나의 콤마 separate된 문자열로 만든뒤 println!의 placeholder인 {}부분에 출력한다.
      println!("  - [{}]", ex_group.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", "));
    }
  }
  // 최종 output 출력을 위한 출력 시작 부분
  println!("--- Output Order (1st: @, 2nd: *) ---");


  // 4. 최종 그룹핑 로직
  // 최종 결과 그룹들을 저장할 빈 벡터를 생성한다.
  let mut result_groups: Vec<Vec<u32>> = Vec::new();
  // 그룹핑 과정에서 이미 처리된 그룹인지를 contains를 이용하여 빠르게 추적하기 위한 HashSet이다.
  let mut processed_docks_in_grouping: HashSet<u32> = HashSet::new();

  // min부터 max까지의 도크들이 담긴 all_docks_in_range를 처음 도크(min)부터 순회한다.
  for &current_dock in &all_docks_in_range {
    // current_dock가 어떤 그룹에 포함되어서 processed_docks_in_grouping에 포함되었다면
    if processed_docks_in_grouping.contains(&current_dock) {
      // 더이상 처리할 필요가 없으므로 continue한다.
      continue;
    }

    // current_dock가 예외 그룹의 시작점인지 판단하기 위한 변수
    let mut is_exception_start = false;
    // 만약 현재 도크가 exception_group의 도크라면 해당 ex_group을 all_exception_docks에서 찾아
    // 여기에 저장한다. 이 data는 optional한 data이다.
    let mut current_exception_group_data: Option<Vec<u32>> = None;

    // current_dock가 전체 예외 도크 Set에 포함됐다면 이 도크는 예외도크이므로
    if all_exception_docks.contains(&current_dock) {
      // final_exception_groups를 순회하며 어떤 예외 도크 그룹에 속하는지 파악한다.
      for ex_g in &final_exception_groups {
        // 만약 current_dock가 ex_g의 first와 일치한다면 이 도크는 ex_g의 도크이므로
        if ex_g.first() == Some(&current_dock) {
          // is_exception_start을 true로 만들고
          is_exception_start = true;
          // current_exception_group_data에 ex_g를 복제하여 넣어놓는다.
          current_exception_group_data = Some(ex_g.clone());
          // 일치하는 예외 그룹을 찾았으므로 루프를 빠져나온다.
          break;
        }
      }
    }

    // 위 과정에서 만약 is_exception_start가 true이고, current_exception_group_data에 무언가 있다면
    if is_exception_start && current_exception_group_data.is_some() {
      // current_exception_group_data에서 ex_group을 추출한뒤
      if let Some(ex_group) = current_exception_group_data {
        // result_groups에 clone하여 push한다.
        result_groups.push(ex_group.clone());
        // 또한 이 ex_group의 dock들을 
        for &dock_in_ex in &ex_group {
          // processed_docks_in_grouping에 insert하여 추후 루핑 과정에서  
          // 이 도크 순서가 온다면 이것을 빠르게 확인하여 건너뛰도록 한다.
          processed_docks_in_grouping.insert(dock_in_ex);
        }
      }
    } // current_dock가 예외 그룹의 시작점이 아니고, 모든 예외 그룹(all_exception_docks)에도 속하지 않는다면
    else if !all_exception_docks.contains(&current_dock) {
      // 새로운 일반 그룹(regular_group)을 생성하고
      let mut regular_group: Vec<u32> = Vec::new();
      // current_dock을 regular_group에 push한다.
      regular_group.push(current_dock);
      // 또한 processed_docks_in_grouping에도 추가하여 processed된 그룹으로 지정한다.
      processed_docks_in_grouping.insert(current_dock);
      
      // priorities HashMap으로 부터 current_dock을 key로 하는 Priority를 얻는다.
      // 만약 이것을 얻을 수 없다면 current_dock_priority는 Priority::Third로 할당된다.
      let current_dock_priority = priorities.get(&current_dock).unwrap_or(&Priority::Third);
      // current_dock_priority를 match하여 각 Priority에 맞는 per_page를 얻은 뒤 변수 current_target_per_page에 할당한다.
      let current_target_per_page = match current_dock_priority {
        Priority::First => fpp,
        Priority::Second => spp,
        Priority::Third => gpp,
      };
      
      // 현재 도크 기준 다음 도크의 index를 찾아 변수에 할당한다.
      let mut next_dock_idx_in_range = all_docks_in_range.iter()
        .position(|&d| d == current_dock).unwrap_or(0) + 1;
      // .position에서 현재 도크와 같은 도크의 index를 찾을 수 없다면 unwrap_or로 안전하게 0을 배출하여 처리한다.
      // 이는 unwrap으로 인한 프로그램 강제 종료를 막기 위함임.

      // --- [그룹 확장 루프] ---
      // 다음 조건들이 모두 만족하는 동안 그룹을 확장합니다:
      // 1. 현재 그룹의 크기가 목표 개수(`current_target_per_page`)보다 작다.
      // 예를들어서 66, 67 도크가 모두 1차 도크이고, 1차 도크의 per-page가 1이라고 하면,
      // 처음 regular_group의 len은 1이고, per-page도 1이다. 따라서 이때 regular_group.len() < current_target_per_page는
      // 1 < 1 => false이므로 while문은 즉시 종료하게 되고, regular_group은 66인 상태로 남게되고, 새로운 67로 시작되는
      // regular_group을 만들게 된다.
      // 반면 51, 52 도크가 1차 2차도 아닌 일반 그룹이라고 하고, per-page가 2라고 하자.
      // 그럼 처음 51 도크가 regular_group에 담기게되고, 이때의 len은 1이다. 그런데 51 도크의 current_taget_per_page는
      // 2 이므로 while문이 진행된다.
      // 2. 확인할 다음 도크가 전체 도크 범위(`all_docks_in_range`) 안에 있다.
      while regular_group.len() < current_target_per_page && next_dock_idx_in_range < all_docks_in_range.len() {
        // current_dock 다음 dock로 지명된 후보이다.
        let next_dock_candidate = all_docks_in_range[next_dock_idx_in_range];

        // [확장 중단 조건 1] next_dock_candidate가 이미 처리된 도크이거나 예외 그룹에 속해있으면 그룹 확장을 중단한다.
        if processed_docks_in_grouping.contains(&next_dock_candidate) || 
          all_exception_docks.contains(&next_dock_candidate) {
          break;
        }

        // [확장 중단 조건 2] 우선순위 규칙 확인
        // 현재 current_dock가 담긴 regular_group의 첫번째 도크의 Priority를 얻는다.
        let regular_group_first_prio = priorities.get(&regular_group[0]).unwrap_or(&Priority::Third);
        // current_dock의 다음인 next_dock_candidate의 Priority를 얻는다.
        let next_candidate_prio = priorities.get(&next_dock_candidate).unwrap_or(&Priority::Third);

        // next_candidate_prio < regular_group_first_prio 조건:
        // 다음 후보 도크의 우선순위가 현재 그룹의 기준 우선순위보다 높으면(enum의 값이 더 작으면) 그룹 확장을 중단합니다.  
        // 예: 3차 그룹(`Third`)을 만드는 중에 1차 도크(`First`)를 만나면, 
        // 1차 도크는 이 그룹에 포함되지 않고 다음 루프에서 자신만의 새 regular 그룹을 시작해야 합니다.
        if /* regular_group_first_prio != next_candidate_prio ||  */ next_candidate_prio < regular_group_first_prio {
          // regular_group_first_prio != next_candidate_prio 조건은 현재 그룹의 우선순위와 동일한 우선순위인 next만 현재 그룹 확장을
          // 허용한다는 조건이다. 따라서 이 조건을 활성화하면 1차 그룹에는 1차만 올수 있고, 3차가 뒤에 붙을 수 없음.
          break;
          //TODO: 1차 그룹 혹은 2차 그룹에 동일 그룹만 오게끔 하거나 혹은 그 하위 그룹이 붙을 수 있게하는 조건 플래그를 하나 만들자.
        }

        // 확장 중단 조건을 모두 통과했다면 regular_group에 next_dock_candidate을 push한다.
        regular_group.push(next_dock_candidate);
        // 또한 next_dock_candidate을 processed에 추가한다.
        processed_docks_in_grouping.insert(next_dock_candidate);
        // 또, next_dock_candidate의 index인 next_dock_idx_in_range를 +1해준다.
        // 만약 next_dock_idx_in_range가 all_docks_in_range.len()를 넘어선다면 while문은 즉시 종료된다.
        next_dock_idx_in_range += 1;
      }
      // 그룹 확장 while 루프가 모두 종료되면 확장이 종료된 regular_group을 result_group에 push한다.
      result_groups.push(regular_group);
    }
  }

  // 5. 결과 출력
  // 최종 결과물인 result_groups를 루핑하여 각 group을 얻는다.
  for group in result_groups {
    // 각 그룹으로부터 1차 2차 기호가 포매팅된 String을 담는 그룹 Vec
    let formatted_group: Vec<String> = group
      .iter()
      .map(|&d| {
        // 현재 도크인 d가 all_exception_docks에 포함된 도크, 즉 예외 그룹이라면
        if all_exception_docks.contains(&d) { 
          // 기호 없이 그대로 String으로 변환한다.
          d.to_string()
        }
        // 예외 도크가 아니라면
        else {
          // priorities에 도크 d를 키로 넣어서 해당 도크의 Priority를 match 시켜서
          match priorities.get(&d) { // 예외가 아닌 도크는 priorities 맵에 우선순위가 있어야 함
            Some(Priority::First) => format!("{d}@"), // 1차 2차에 맞는 기호를 붙여준다.
            Some(Priority::Second) => format!("{d}*"),
            Some(Priority::Third) | None => d.to_string(), // 3차 또는 Priority가 None일때에는 
            // 그대로 넣는다.
          }
        }
      })
      .collect();
    // 최종적으로 formatted_group을 join을 이용하여 comma separator로 구분하여 출력해준다.
    println!("{}", formatted_group.join(", "));
  }
}
