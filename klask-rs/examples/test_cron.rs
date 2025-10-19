use chrono::{Timelike, Utc};
use croner::Cron;

fn main() {
    let now = Utc::now();
    println!("Current time: {}", now);
    println!("Hour: {}, Minute: {}", now.hour(), now.minute());

    let cron_expr = "0 0 1 * * *";
    println!("\nTesting cron expression: {}", cron_expr);
    println!("Expected: Should run at 01:00:00 every day");

    match cron_expr.parse::<Cron>() {
        Ok(cron) => {
            match cron.find_next_occurrence(&now, false) {
                Ok(next) => {
                    println!("\nNext run time: {}", next);
                    println!("Next run hour: {}, minute: {}", next.hour(), next.minute());
                    let duration = next - now;
                    let hours = duration.num_hours();
                    let minutes = duration.num_minutes() % 60;
                    println!("Time until next run: {}h {}m", hours, minutes);

                    // If it's past 1am, should be tomorrow at 1am (about 23 hours away)
                    // If it's before 1am, should be today at 1am
                    if now.hour() >= 1 {
                        println!("\nExpected: Should be tomorrow at 01:00 (roughly 23h away)");
                    } else {
                        println!("\nExpected: Should be today at 01:00");
                    }
                }
                Err(e) => println!("Error finding next occurrence: {}", e),
            }
        }
        Err(e) => println!("Error parsing cron: {}", e),
    }

    // Test a few more expressions
    println!("\n--- Testing other expressions ---");

    let test_cases = vec![
        ("0 */5 * * * *", "Every 5 minutes"),
        ("0 0 */6 * * *", "Every 6 hours"),
        ("0 30 14 * * *", "Every day at 14:30"),
    ];

    for (expr, description) in test_cases {
        println!("\nExpression: {} ({})", expr, description);
        if let Ok(cron) = expr.parse::<Cron>() {
            if let Ok(next) = cron.find_next_occurrence(&now, false) {
                let duration = next - now;
                println!("Next: {} (in {}m)", next, duration.num_minutes());
            }
        }
    }
}
