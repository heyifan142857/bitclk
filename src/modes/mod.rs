pub mod clock;
pub mod stopwatch;
pub mod timer;

pub fn placeholder_message(mode: &str) -> String {
    format!(
        "bitclk {mode} is not implemented yet.\nclock is ready today; {mode} will land in a future release."
    )
}
