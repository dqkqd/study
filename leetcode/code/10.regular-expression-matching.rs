enum Match {
    One(Elem),
    ManyOrNone(Elem),
}
enum Elem {
    Char(char),
    Any,
}
impl From<char> for Elem {
    fn from(value: char) -> Elem {
        if value == '.' {
            Elem::Any
        } else {
            Elem::Char(value)
        }
    }
}
impl Elem {
    fn equal(&self, c: &char) -> bool {
        match self {
            Elem::Char(v) => v == c,
            Elem::Any => true,
        }
    }
}

impl Solution {
    pub fn is_match(s: String, p: String) -> bool {
        let pchars: Vec<char> = p.chars().collect();

        let mut pmatch = vec![];
        for w in pchars.windows(2) {
            if w[0] == '*' {
                continue;
            }
            if w[1] == '*' {
                pmatch.push(Match::ManyOrNone(Elem::from(w[0])));
            } else {
                pmatch.push(Match::One(Elem::from(w[0])));
            }
        }
        if let Some(c) = pchars.last() {
            if c != &'*' {
                pmatch.push(Match::One(Elem::from(*c)));
            }
        }

        let schars: Vec<char> = s.chars().collect();
        let mut dp = vec![vec![false; pmatch.len() + 1]; schars.len() + 1];
        dp[0][0] = true;
        for (pi, m) in pmatch.iter().enumerate() {
            match m {
                Match::One(_) => dp[0][pi + 1] = false,
                Match::ManyOrNone(_) => dp[0][pi + 1] = dp[0][pi],
            }
        }

        for (si, c) in schars.iter().enumerate() {
            for (pi, m) in pmatch.iter().enumerate() {
                match m {
                    Match::One(elem) => {
                        // must use
                        if dp[si][pi] && elem.equal(c) {
                            dp[si + 1][pi + 1] = true;
                        }
                    }
                    Match::ManyOrNone(elem) => {
                        // use it
                        if (dp[si][pi] || dp[si][pi + 1]) && elem.equal(c) {
                            dp[si + 1][pi + 1] = true;
                        }
                        // or not
                        if dp[si + 1][pi] {
                            dp[si + 1][pi + 1] = true;
                        }
                    }
                }
            }
        }

        dp[schars.len()][pmatch.len()]
    }
}
