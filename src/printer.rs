use crate::{cli::Args, models::Priority, processor::ProcessingResult};


pub fn print_results(args: &Args, result_data: &ProcessingResult) {
  // 처리 도크의 min..max 도크 range를 출력한다.
  println!("Processing dock range: {} - {}", args.min, args.max);
  // 1차, 2차 그룹, 일반 그룹의 각 처리당 per-page들을 출력한다.
  println!("Docks per group (1st priority): {}", result_data.fpp);
  println!("Docks per group (2nd priority): {}", result_data.spp);
  println!("Docks per group (3rd priority/general): {}", result_data.gpp);
  // 만약 strict mode가 적용되었다면 모드 적용이 됐음을 출력한다.
  if args.strict_first {
    println!("Strict mode applyed for 1st priority groups.");
  }
  if args.strict_second {
    println!("Strict mode applyed for 2nd priority groups.");
  }

  // 만약 final_exception_groups이 있는 경우 해당 그룹들을 출력해준다.
  if !result_data.final_exception_groups.is_empty() {
    println!("Exception groups (printed together, in order of their first dock):");
    // final_exception_groups의 각 그룹들을 순회한다.
    for ex_group in &result_data.final_exception_groups {
      // 각 ex_group을 iter().map하여 각 dock인 d를 string으로 만든뒤 이것을 다시 Vec으로 collect한뒤 이 Vec을 
      // join을 이용하여 하나의 콤마 separate된 문자열로 만든뒤 println!의 placeholder인 {}부분에 출력한다.
      println!("  - [{}]", ex_group.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", "));
    }
  }
  // 최종 output 출력을 위한 출력 시작 부분
  println!("--- Output Order (1st: @, 2nd: *) ---");

  // 5. 결과 출력
  // 최종 결과물인 result_groups를 루핑하여 각 group을 얻는다.
  for group in &result_data.result_groups {
    // 각 그룹으로부터 1차 2차 기호가 포매팅된 String을 담는 그룹 Vec
    let formatted_group: Vec<String> = group
      .iter()
      .map(|&d| {
        // 현재 도크인 d가 all_exception_docks에 포함된 도크, 즉 예외 그룹이라면
        if result_data.all_exception_docks.contains(&d) { 
          // 기호 없이 그대로 String으로 변환한다.
          d.to_string()
        }
        // 예외 도크가 아니라면
        else {
          // print_marker flag가 설정되었다면
          if args.print_marker {
            // priorities에 도크 d를 키로 넣어서 해당 도크의 Priority를 match 시켜서
            match result_data.priorities.get(&d) {
              // 각 Priority에 맞는 기호를 붙여 출력한다.
              Some(Priority::First) => format!("{d}@"),
              Some(Priority::Second) => format!("{d}*"),
              Some(Priority::Third) => d.to_string(),
              None => d.to_string()
            }
          // print_marker가 Set되지 않았다면 그냥 출력한다.
          } else {
            d.to_string()
          }
        }
      })
      .collect();
    // 최종적으로 formatted_group을 join을 이용하여 comma separator로 구분하여 출력해준다.
    println!("{}", formatted_group.join(", "));
  }
}