use chrono::{Timelike, Utc};
use croner::Cron;

fn main() {
    let now = Utc::now();
    println!("Current time UTC: {}", now);
    println!("Hour: {}, Minute: {}", now.hour(), now.minute());

    let cron_expr = "0 0 22 * * *";
    println!("\nTesting cron expression: {}", cron_expr);
    println!("Expected: Should run at 22:00:00 UTC every day");

    match cron_expr.parse::<Cron>() {
        Ok(cron) => match cron.find_next_occurrence(&now, false) {
            Ok(next) => {
                println!("\nNext run time: {}", next);
                println!("Next run hour: {}, minute: {}", next.hour(), next.minute());
                let duration = next - now;
                let hours = duration.num_hours();
                let minutes = duration.num_minutes() % 60;
                println!("Time until next run: {}h {}m", hours, minutes);

                println!("\nIn your local timezone (CEST = UTC+2):");
                println!("22:00 UTC = 00:00 (midnight) local time");
                if now.hour() < 22 {
                    println!("Next run: Tonight at midnight local time");
                } else {
                    println!("Next run: Tomorrow night at midnight local time");
                }
            }
            Err(e) => println!("Error finding next occurrence: {}", e),
        },
        Err(e) => println!("Error parsing cron: {}", e),
    }
}
