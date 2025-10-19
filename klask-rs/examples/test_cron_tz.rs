use chrono::{Timelike, Utc};
use croner::Cron;

fn main() {
    println!("=== Testing Cron with Timezones ===\n");

    // Current time in UTC
    let now_utc = Utc::now();
    println!("Current time UTC: {}", now_utc);
    println!("Hour: {}, Minute: {}", now_utc.hour(), now_utc.minute());

    // Test cron expression: 0 0 1 * * * (1am)
    let cron_expr = "0 0 1 * * *";
    println!("\n--- Testing cron: {} (every day at 1am) ---", cron_expr);

    // Parse cron
    let cron = cron_expr.parse::<Cron>().expect("Failed to parse cron");

    // Find next occurrence using UTC
    println!("\n1. Using UTC timezone:");
    match cron.find_next_occurrence(&now_utc, false) {
        Ok(next) => {
            println!("   Next run (UTC): {}", next);
            println!("   Next run hour: {}", next.hour());
            let duration = next - now_utc;
            let hours = duration.num_hours();
            let minutes = duration.num_minutes() % 60;
            println!("   Time until next run: {}h {}m", hours, minutes);
        }
        Err(e) => println!("   Error: {}", e),
    }

    println!("\n=== Explanation ===");
    println!("The cron expression '0 0 1 * * *' means 1am UTC");
    println!("Note: To use different timezones, the chrono-tz crate needs to be added as a dependency");
    println!("\nFor users to set schedules in their local time, we need to:");
    println!("1. Let them specify their timezone in the UI");
    println!("2. Convert the cron expression to their timezone before calculating next run");
}
