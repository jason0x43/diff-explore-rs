use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::process::Command;

#[derive(Serialize, Deserialize)]
pub struct Commit {
    pub commit: String,
    pub decoration: String,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: u64,
    pub subject: String,
}

pub fn git_log<'a>() -> Result<Vec<Commit>> {
    let output = Command::new("git")
        .arg("log")
        .arg("--date=iso8601-strict")
        .arg("--decorate")
        .arg(
            "--pretty=format:{\
			  \"commit\":\"%H\",\
			  \"decoration\":\"%d\",\
			  \"author_name\":\"%aN\",\
			  \"author_email\":\"%aE\",\
			  \"timestamp\":%at,\
			  \"subject\":\"%f\"\
			},",
        )
        .output()
        .expect("unable to read git log");

    let out_str =
        String::from_utf8(output.stdout).expect("invalid output string");
    let out_clean = out_str.trim_end_matches("\n").trim_end_matches(",");
    let out_array = format!("[{}]", out_clean);
    return serde_json::from_str(&out_array);
}
