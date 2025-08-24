use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Dock Label Output Order and Range Calculator", long_about = None)]
pub struct Args {
  /// First priority docks. Can be single numbers or ranges (e.g., 1-3 5 7-9)
  #[arg(short = 'f', long, value_delimiter = ' ', num_args = 0.., required = false, value_parser = parse_dock_ranges, action = clap::ArgAction::Append)]
  pub first_priority: Vec<Vec<u32>>, // clap이 Vec<Vec<u32>>를 만들도록 하고, 나중에 flatten
  // 예를들어서 -f 65-66 71 56 62 이런식으로 입력됐다면,
  // parse_dock_ranges 함수에 의해 각각 [[65, 66], [71], [56], [62]] 이런식으로 리스트가 만들어진다.
  /// Second priority docks. Can be single numbers or ranges (e.g., 10-12 15)
  #[arg(short = 's', long, value_delimiter = ' ', num_args = 0.., required = false, value_parser = parse_dock_ranges, action = clap::ArgAction::Append)]
  pub second_priority: Vec<Vec<u32>>, // clap이 Vec<Vec<u32>>를 만들도록 하고, 나중에 flatten

  /// Exception docks to be grouped together, ignoring -p. (e.g., 1-3 7-9 10)
  #[arg(long = "exceptions", short = 'e', value_delimiter = ' ', num_args = 0.., required = false, value_parser = parse_dock_ranges, action = clap::ArgAction::Append)]
  pub exception_groups_raw: Vec<Vec<u32>>, // 각 예외 그룹을 Vec<u32>로 받음
  // 예외 그룹은 1-3 같은 연속 범위나 10 같은 단일 그룹으로 지정될 수 있다.
  // _raw는 flatten되지 않은 [[1, 2, 3], [10]] 같은 형식의 Vec이다.
  /// Number of docks to print per group
  #[arg(short = 'p', long)]
  pub per_page: u16,

  /// Number of docks per group for 1st priority docks (defaults to -p value if not set)
  #[arg(short = '1', long = "fp", required = false)] // short: -1, long: --fpp
  pub first_priority_per_page: Option<u16>,

  /// Number of docks per group for 2nd priority docks (defaults to -p value if not set)
  #[arg(short = '2', long = "sp", required = false)] // short: -2, long: --spp
  pub second_priority_per_page: Option<u16>,

  /// Minimum dock number to process
  #[arg(long, required = false, default_value_t = 51)] // 기본값 51로 설정, optional
  pub min: u32,

  /// Maximum dock number to process
  #[arg(long, required = false, default_value_t = 78)] // 기본값 78로 설정, optional
  pub max: u32,

  // 그룹 확장 조건을 더 엄격하게 하는 플래그이다. 이 플래그가 입력되면
  // 1차 그룹은 1차 그룹끼리만 그루핑된다. 플래그가 입력되지 않으면 1차 그룹 뒤에 하위 그룹 도크들이 붙을 수 있다.
  /// Group 1st priority docks strictly with other 1st priority docks only.
  ///
  /// When this flag is not set, lower priority docks can be appended to a 1st priority group.
  #[arg(long = "strict-first", short = 'F', action = clap::ArgAction::SetTrue)]
  pub strict_first: bool,

  // 2차 그룹 끼리만 엄격히 묶는 플래그. 윗 플래그와 동일한 기능이다.
  /// Group 2nd priority docks strictly with other 2nd priority docks only.
  ///
  /// When this flag is not set, 3rd priority docks can be appended to a 2nd priority group.
  #[arg(long = "strict-second", short = 'S', action = clap::ArgAction::SetTrue)]
  pub strict_second: bool,

  // 1차, 2차 도크에 marker를 출력하는지 여부의 플래그
  /// Print markers ('@' for 1st, '*' for 2nd) next to priority dock numbers.
  #[arg(long = "mark", short = 'm', action = clap::ArgAction::SetTrue)]
  pub print_marker: bool,
}

impl Args {
  pub fn validate_input(&self) -> Result<(), String> {
    if self.per_page == 0 {
      return Err(
        "Error: Number of docks per group must be 1 or greater for all per-page settings."
          .to_string(),
      );
    }
    if self.first_priority_per_page == Some(0) {
      return Err("Number of docks for 1st priority (`--fpp`) must be 1 or greater.".to_string());
    }
    if self.second_priority_per_page == Some(0) {
      return Err("Number of docks for 2nd priority (`--spp`) must be 1 or greater.".to_string());
    }

    // min과 max를 비교하여 min이 max보다 큰 경우
    if self.min > self.max {
      return Err(format!(
        "Minimum dock number ({}) cannot be greater than maximum dock number ({}).",
        self.min, self.max
      ));
    }

    Ok(())
  }
}

/// 입력된 문자열(단일 숫자 또는 "숫자-숫자" 범위)을 파싱하여 u32의 Vec으로 변환하는 함수.
/// clap의 value_parser로 사용됩니다.
pub fn parse_dock_ranges(s: &str) -> Result<Vec<u32>, String> {
  // 파싱된 도크 숫자들이 저장될 Vec
  let mut docks = Vec::new();

  if s.contains('-') {
    // 만약 arg가 `-`를 포함한다면
    // 두 개의 숫자로 split 한다.
    let parts: Vec<&str> = s.splitn(2, '-').collect();
    if parts.len() == 2 {
      // split된 parts가 2개라면
      let start_str = parts[0].trim(); // parts[0]을 trim하여 start_str에 저장한다.
      let end_str = parts[1].trim(); // 마찬가지로 parts[1]을 trim하여 end_str에 저장한다.
      // 만약 start_str와 end_str를 u32로 파싱하는게 Ok라면 파싱된 값을 start와 end에 할당한다.
      if let (Ok(start), Ok(end)) = (start_str.parse::<u32>(), end_str.parse::<u32>()) {
        if start <= end {
          // start가 end보다 작거나 같다면
          for i in start..=end {
            // start에서 시작하여 end를 포함하여 범위를 생성하고
            docks.push(i); // 범위에서 생성된 숫자 i를 docks에 push한다.
          }
        } else {
          // start가 end보다 큰 경우
          return Err(format!(
            // 에러 메세지를 내뱉는다.
            "Invalid range: start ({start}) must be less than or equal to end ({end}) in '{s}'"
          ));
        }
      } else {
        // u32 파싱에 실패한경우. 입력된 문자열이 숫자 형식이 아니라서 발생할 수 있음.
        return Err(format!(
          "Invalid range format: '{s}'. Both parts must be numbers."
        ));
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
