#[macro_export]
macro_rules! string_enum {
    (pub enum $name:ident { $($variant:ident => $str:expr,)* }) => {
        #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
        #[serde(rename_all = "kebab-case")]
        pub enum $name {
            $($variant,)*
        }

        impl $name {
            pub fn to_str(&self) -> &'static str {
                match self {
                    $($name::$variant => $str,)*
                }
            }
        }

        impl ::std::str::FromStr for $name {
            type Err = ::anyhow::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $($str => Ok($name::$variant),)*
                    other => ::anyhow::bail!("invalid {}: {}", stringify!($name), other),
                }
            }
        }

        impl ::std::fmt::Display for $name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "{}", self.to_str())
            }
        }
    };
}

#[macro_export]
macro_rules! from_into_string {
    ($type:ty) => {
        impl From<$type> for String {
            fn from(val: $type) -> String {
                val.to_string()
            }
        }

        impl TryFrom<String> for $type {
            type Error = ::anyhow::Error;

            fn try_from(s: String) -> Result<Self, Self::Error> {
                s.parse()
            }
        }
    };
}
