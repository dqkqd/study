use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Clock {
    hours: i32,
    minutes: i32,
}

impl Clock {
    pub fn new(hours: i32, minutes: i32) -> Self {
        Clock { hours, minutes }.normalize()
    }

    fn normalize(&self) -> Self {
        let mut total_minutes = 60 * (self.hours as i64) + self.minutes as i64;
        total_minutes %= 24 * 60;
        total_minutes += 24 * 60;
        total_minutes %= 24 * 60;

        Clock {
            hours: (total_minutes / 60) as i32,
            minutes: (total_minutes % 60) as i32,
        }
    }

    pub fn add_minutes(&self, minutes: i32) -> Self {
        Clock {
            hours: self.hours,
            minutes: self.minutes + minutes,
        }
        .normalize()
    }
}

impl PartialEq for Clock {
    fn eq(&self, other: &Self) -> bool {
        let lhs = self.normalize();
        let rhs = other.normalize();
        dbg!(&lhs, &rhs);
        dbg!(Clock::new(-1, 0));
        lhs.hours == rhs.hours && lhs.minutes == rhs.minutes
    }
}

impl Display for Clock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}", self.hours, self.minutes)
    }
}
