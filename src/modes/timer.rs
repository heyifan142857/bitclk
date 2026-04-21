use crate::app::AppResult;
use crate::modes::placeholder_message;

pub fn run() -> AppResult {
    println!("{}", message());
    Ok(())
}

fn message() -> String {
    placeholder_message("timer")
}

#[cfg(test)]
mod tests {
    use super::message;

    #[test]
    fn placeholder_message_is_friendly() {
        let message = message();

        assert!(message.contains("not implemented yet"));
        assert!(message.contains("timer"));
    }
}
