use std::collections::{HashMap, HashSet};

use crate::{cli::Args, models::Priority};

pub struct ProcessingResult {
  pub result_groups: Vec<Vec<u32>>,
  pub priorities: HashMap<u32, Priority>,
  pub all_exception_docks: HashSet<u32>,
  pub fpp: u16,
  pub spp: u16,
  pub gpp: u16,
  pub final_exception_groups: Vec<Vec<u32>>,
}

pub fn process_docks(args: &Args) -> ProcessingResult {

  // per_page 값 결정 로직
  // first와 second는 optional한 값이므로 값이 없다면 per_page를 따르도록 한다.
  let fpp = args.first_priority_per_page.unwrap_or(args.per_page);
  let spp = args.second_priority_per_page.unwrap_or(args.per_page);
  let gpp = args.per_page; // general per page(third)

  // 1. 입력된 우선순위 및 예외 도크 정리
  // -f 65-66 71 56 62 와 같이 입력했다면 [[65, 66], [71], [56], [62]] 이런식인데, 이걸 flatten을 이용해서
  // [65, 66, 71, 56, 62] 이렇게 만들어 HashSet에 저장해준다.
  let first_priority_docks: HashSet<u32> = args.first_priority.clone().into_iter().flatten().collect();
  let second_priority_docks: HashSet<u32> = args.second_priority.clone().into_iter().flatten().collect();
  
  // 예외 그룹 처리: 각 예외 그룹을 정렬하고, 전체 예외 도크 집합을 만듦.
  // 최종적인 exception_group Vec들이 들어갈 Vec이다.
  let mut final_exception_groups: Vec<Vec<u32>> = Vec::new();
  // args.exception_groups_raw에서의 모든 예외 도크들을 담는 HashSet.
  let mut all_exception_docks: HashSet<u32> = HashSet::new();
  // 범위 밖을 벗어난 입력값이 있다면 해당 값을 경고 메세지에 지정한 뒤 경고 메세지들을 저장하여 나중에 출력하기 위한 Vec다.
  let mut warnings: Vec<String> = Vec::new();

  // args에서 exception_groups_raw에 접근하여 각 raw_ex_group Vec을 순회한다.
  for raw_ex_group in &args.exception_groups_raw {
    // raw_ex_group에서 각 숫자들을 검사하여 min과 max 사이의 값인지를 필터링하여 current_ex_group을 얻는다.
    let mut current_ex_group: Vec<u32> = raw_ex_group.iter()
      .filter(|&d| {
        // raw_ex_group의 각 숫자가 min과 max 사이의 값인지를 필터링한다.
        if d >= &args.min && d <= &args.max { true } 
        else { // min max 값 이외의 범위에 있는 숫자라면 ignored되고 해당 숫자는 경고 메세지에 저장되어 
          // 이 메세지를 warnings에 담아둔다.
          warnings.push(
            format!("Warning: Exception dock {} is outside the specified range [{}-{}] and will be ignored.", 
              d, args.min, args.max));
          false // 이 경우에는 false로 처리하여 필터링한다.
        }
      }).copied().collect();
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
    if dock >= args.min && dock <= args.max && !all_exception_docks.contains(&dock) {
      // 해당 dock를 priorites HashMap에 dock를 key로, Priority::First를 value로 insert한다.
      priorities.insert(dock, Priority::First);
    } // 그게 아니라 min max 범위를 벗어난 값이 있다면
    else if !(dock >= args.min && dock <= args.max) { // 범위 밖 경고
      // warnings에 해당 dock의 경고 메세지를 저장한다.
      warnings.push(format!(
        "Warning: First priority dock {} is outside the specified range [{}-{}] and will be ignored.",
        dock, args.min, args.max
      ));
    }
  }

  // 2차 그룹도 1차 그룹과 같은 방식으로 처리한다.
  for &dock in &second_priority_docks {
    if dock >= args.min && dock <= args.max && !all_exception_docks.contains(&dock) {
      // 이 경우에는 Priority::Second를 값으로 넣어둔다.
      priorities.entry(dock).or_insert(Priority::Second);
    } else if !(dock >= args.min && dock <= args.max) { // 범위 밖 경고
       warnings.push(format!(
        "Warning: Second priority dock {} is outside the specified range [{}-{}] and will be ignored.",
        dock, args.min, args.max
      ));
    }
  }
  // 3차 우선순위는 나중에 그룹핑 시점에 기본값으로 처리한다.

  // 경고 메시지를 출력한다.
  for warning in warnings {
    eprintln!("{warning}");
  }

  // 처리할 전체 도크 목록 = min부터 max까지의 처리할 모든 도크가 담긴 Vec이다.
  let all_docks_in_range: Vec<u32> = (args.min..=args.max).collect();

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
      // --1. 현재 그룹의 크기가 목표 개수(`current_target_per_page`)보다 작다.
      // 예를들어서 66, 67 도크가 모두 1차 도크이고, 1차 도크의 per-page가 1이라고 하면,
      // 처음 regular_group의 len은 1이고, per-page도 1이다. 따라서 이때 regular_group.len() < current_target_per_page는
      // 1 < 1 => false이므로 while문은 즉시 종료하게 되고, regular_group은 66인 상태로 남게되고, 새로운 67로 시작되는
      // regular_group을 만들게 된다.
      // 반면 51, 52 도크가 1차 2차도 아닌 일반 그룹이라고 하고, per-page가 2라고 하자.
      // 그럼 처음 51 도크가 regular_group에 담기게되고, 이때의 len은 1이다. 그런데 51 도크의 current_taget_per_page는
      // 2 이므로 while문이 진행된다.
      // --2. 확인할 다음 도크가 전체 도크 범위(`all_docks_in_range`) 안에 있다.
      while regular_group.len() < current_target_per_page.into() && next_dock_idx_in_range < all_docks_in_range.len() {
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

        // 확장 중단 조건 2 규칙의 결과에 따라 break를 결정하기 위한 bool 변수
        let should_break = 
          // 첫번째 조건: next 도크의 우선순위가 현재 도크의 우선순위보다 낮은 경우
          // 예를들면 3차 도크 뒤에 1차 도크가 오는 경우 break하고 새로운 1차 도크의 regular_group을 만들어야한다.
          (next_candidate_prio < regular_group_first_prio) ||
          // 만약 strict_first와 같은 플래그가 설정됐다면, 1차 그룹은 1차 그룹끼리만 묶여진다. 즉, next가 1차 그룹이 
          // 아니라면 즉시 break 되어 새로운 regular_group을 생성해야한다.
          (*regular_group_first_prio == Priority::First && 
            args.strict_first && *next_candidate_prio != Priority::First) ||
          // 2차 그룹 역시 strict mode 플래그에 따라 해당 조건이 활성화된다. 
          (*regular_group_first_prio == Priority::Second && 
            args.strict_second && *next_candidate_prio != Priority::Second);

        // 확장 중단 조건 2의 결과에 따라 break를 할지 말지가 결정된다.
        if should_break {
          break;
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

  ProcessingResult { result_groups, priorities, all_exception_docks, fpp, spp, gpp, final_exception_groups }
}