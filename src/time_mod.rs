#[cfg(not(feature = "std"))]
use crate::no_std_prelude::*;
#[cfg(feature = "std")]
use crate::PrimitiveDateTime;
use crate::{
    format::{parse, parse::AmPm, ParseError, ParseResult, ParsedItems},
    ComponentRangeError, DeferredFormat, Duration,
};
use core::{
    cmp::Ordering,
    num::NonZeroU8,
    ops::{Add, AddAssign, Sub, SubAssign},
    time::Duration as StdDuration,
};
use time_macros::time;

/// The number of nanoseconds in one day.
pub(crate) const NANOS_PER_DAY: u64 = 24 * 60 * 60 * 1_000_000_000;

/// The clock time within a given date. Nanosecond precision.
///
/// All minutes are assumed to have exactly 60 seconds; no attempt is made to
/// handle leap seconds (either positive or negative).
///
/// When comparing two `Time`s, they are assumed to be in the same calendar
/// date.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(from = "crate::serde::Time", into = "crate::serde::Time")
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Time {
    #[allow(clippy::missing_docs_in_private_items)]
    pub(crate) hour: u8,
    #[allow(clippy::missing_docs_in_private_items)]
    pub(crate) minute: u8,
    #[allow(clippy::missing_docs_in_private_items)]
    pub(crate) second: u8,
    #[allow(clippy::missing_docs_in_private_items)]
    pub(crate) nanosecond: u32,
}

impl Time {
    /// Create a `Time` that is exactly midnight.
    ///
    /// ```rust
    /// # use time::{Time, time};
    /// assert_eq!(Time::midnight(), time!(0:00));
    /// ```
    #[inline(always)]
    pub const fn midnight() -> Self {
        time!(0:00)
    }

    /// Create a `Time` from the hour, minute, and second.
    ///
    /// ```rust
    /// # use time::Time;
    /// let time = Time::from_hms(1, 2, 3);
    /// assert_eq!(time.hour(), 1);
    /// assert_eq!(time.minute(), 2);
    /// assert_eq!(time.second(), 3);
    /// assert_eq!(time.nanosecond(), 0);
    /// ```
    ///
    /// Panics if any component is not valid.
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms(24, 0, 0); // 24 isn't a valid hour.
    /// ```
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms(0, 60, 0); // 60 isn't a valid minute.
    /// ```
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms(0, 0, 60); // 60 isn't a valid second.
    /// ```
    #[inline(always)]
    #[cfg(feature = "panicking-api")]
    #[cfg_attr(doc, doc(cfg(feature = "panicking-api")))]
    #[deprecated(
        since = "0.2.3",
        note = "For times knowable at compile-time, use the `time!` macro. For situations where a \
                value isn't known, use `Time::try_from_hms`."
    )]
    pub fn from_hms(hour: u8, minute: u8, second: u8) -> Self {
        assert_value_in_range!(hour in 0 => 23);
        assert_value_in_range!(minute in 0 => 59);
        assert_value_in_range!(second in 0 => 59);
        Self {
            hour,
            minute,
            second,
            nanosecond: 0,
        }
    }

    /// Attempt to create a `Time` from the hour, minute, and second.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::try_from_hms(1, 2, 3).is_ok());
    /// ```
    ///
    /// Returns `None` if any component is not valid.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::try_from_hms(24, 0, 0).is_err()); // 24 isn't a valid hour.
    /// assert!(Time::try_from_hms(0, 60, 0).is_err()); // 60 isn't a valid minute.
    /// assert!(Time::try_from_hms(0, 0, 60).is_err()); // 60 isn't a valid second.
    /// ```
    #[inline(always)]
    pub fn try_from_hms(hour: u8, minute: u8, second: u8) -> Result<Self, ComponentRangeError> {
        ensure_value_in_range!(hour in 0 => 23);
        ensure_value_in_range!(minute in 0 => 59);
        ensure_value_in_range!(second in 0 => 59);
        Ok(Self {
            hour,
            minute,
            second,
            nanosecond: 0,
        })
    }

    /// Create a `Time` from the hour, minute, second, and millisecond.
    ///
    /// ```rust
    /// # use time::Time;
    /// let time = Time::from_hms_milli(1, 2, 3, 4);
    /// assert_eq!(time.hour(), 1);
    /// assert_eq!(time.minute(), 2);
    /// assert_eq!(time.second(), 3);
    /// assert_eq!(time.millisecond(), 4);
    /// assert_eq!(time.nanosecond(), 4_000_000);
    /// ```
    ///
    /// Panics if any component is not valid.
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_milli(24, 0, 0, 0); // 24 isn't a valid hour.
    /// ```
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_milli(0, 60, 0, 0); // 60 isn't a valid minute.
    /// ```
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_milli(0, 0, 60, 0); // 60 isn't a valid second.
    /// ```
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_milli(0, 0, 0, 1_000); // 1_000 isn't a valid millisecond.
    /// ```
    #[inline(always)]
    #[cfg(feature = "panicking-api")]
    #[cfg_attr(doc, doc(cfg(feature = "panicking-api")))]
    #[deprecated(
        since = "0.2.3",
        note = "For times knowable at compile-time, use the `time!` macro. For situations where a \
                value isn't known, use `Time::try_from_hms_milli`."
    )]
    pub fn from_hms_milli(hour: u8, minute: u8, second: u8, millisecond: u16) -> Self {
        assert_value_in_range!(hour in 0 => 23);
        assert_value_in_range!(minute in 0 => 59);
        assert_value_in_range!(second in 0 => 59);
        assert_value_in_range!(millisecond in 0 => 999);
        Self {
            hour,
            minute,
            second,
            nanosecond: millisecond as u32 * 1_000_000,
        }
    }

    /// Attempt to create a `Time` from the hour, minute, second, and millisecond.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::try_from_hms_milli(1, 2, 3, 4).is_ok());
    /// ```
    ///
    /// Returns `None` if any component is not valid.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::try_from_hms_milli(24, 0, 0, 0).is_err()); // 24 isn't a valid hour.
    /// assert!(Time::try_from_hms_milli(0, 60, 0, 0).is_err()); // 60 isn't a valid minute.
    /// assert!(Time::try_from_hms_milli(0, 0, 60, 0).is_err()); // 60 isn't a valid second.
    /// assert!(Time::try_from_hms_milli(0, 0, 0, 1_000).is_err()); // 1_000 isn't a valid millisecond.
    /// ```
    #[inline(always)]
    pub fn try_from_hms_milli(
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
    ) -> Result<Self, ComponentRangeError> {
        ensure_value_in_range!(hour in 0 => 23);
        ensure_value_in_range!(minute in 0 => 59);
        ensure_value_in_range!(second in 0 => 59);
        ensure_value_in_range!(millisecond in 0 => 999);
        Ok(Self {
            hour,
            minute,
            second,
            nanosecond: millisecond as u32 * 1_000_000,
        })
    }

    /// Create a `Time` from the hour, minute, second, and microsecond.
    ///
    /// ```rust
    /// # use time::Time;
    /// let time = Time::from_hms_micro(1, 2, 3, 4);
    /// assert_eq!(time.hour(), 1);
    /// assert_eq!(time.minute(), 2);
    /// assert_eq!(time.second(), 3);
    /// assert_eq!(time.microsecond(), 4);
    /// assert_eq!(time.nanosecond(), 4_000);
    /// ```
    ///
    /// Panics if any component is not valid.
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_micro(24, 0, 0, 0); // 24 isn't a valid hour.
    /// ```
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_micro(0, 60, 0, 0); // 60 isn't a valid minute.
    /// ```
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_micro(0, 0, 60, 0); // 60 isn't a valid second.
    /// ```
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_micro(0, 0, 0, 1_000_000); // 1_000_000 isn't a valid microsecond.
    /// ```
    #[inline(always)]
    #[cfg(feature = "panicking-api")]
    #[cfg_attr(doc, doc(cfg(feature = "panicking-api")))]
    #[deprecated(
        since = "0.2.3",
        note = "For times knowable at compile-time, use the `time!` macro. For situations where a \
                value isn't known, use `Time::try_from_hms_micro`."
    )]
    pub fn from_hms_micro(hour: u8, minute: u8, second: u8, microsecond: u32) -> Self {
        assert_value_in_range!(hour in 0 => 23);
        assert_value_in_range!(minute in 0 => 59);
        assert_value_in_range!(second in 0 => 59);
        assert_value_in_range!(microsecond in 0 => 999_999);
        Self {
            hour,
            minute,
            second,
            nanosecond: microsecond * 1_000,
        }
    }

    /// Attempt to create a `Time` from the hour, minute, second, and microsecond.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::try_from_hms_micro(1, 2, 3, 4).is_ok());
    /// ```
    ///
    /// Returns `None` if any component is not valid.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::try_from_hms_micro(24, 0, 0, 0).is_err()); // 24 isn't a valid hour.
    /// assert!(Time::try_from_hms_micro(0, 60, 0, 0).is_err()); // 60 isn't a valid minute.
    /// assert!(Time::try_from_hms_micro(0, 0, 60, 0).is_err()); // 60 isn't a valid second.
    /// assert!(Time::try_from_hms_micro(0, 0, 0, 1_000_000).is_err()); // 1_000_000 isn't a valid microsecond.
    /// ```
    #[inline(always)]
    pub fn try_from_hms_micro(
        hour: u8,
        minute: u8,
        second: u8,
        microsecond: u32,
    ) -> Result<Self, ComponentRangeError> {
        ensure_value_in_range!(hour in 0 => 23);
        ensure_value_in_range!(minute in 0 => 59);
        ensure_value_in_range!(second in 0 => 59);
        ensure_value_in_range!(microsecond in 0 => 999_999);
        Ok(Self {
            hour,
            minute,
            second,
            nanosecond: microsecond * 1_000,
        })
    }

    /// Create a `Time` from the hour, minute, second, and nanosecond.
    ///
    /// ```rust
    /// # use time::Time;
    /// let time = Time::from_hms_nano(1, 2, 3, 4);
    /// assert_eq!(time.hour(), 1);
    /// assert_eq!(time.minute(), 2);
    /// assert_eq!(time.second(), 3);
    /// assert_eq!(time.nanosecond(), 4);
    /// ```
    ///
    /// Panics if any component is not valid.
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_nano(24, 0, 0, 0); // 24 isn't a valid hour.
    /// ```
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_nano(0, 60, 0, 0); // 60 isn't a valid minute.
    /// ```
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_nano(0, 0, 60, 0); // 60 isn't a valid second.
    /// ```
    ///
    /// ```rust,should_panic
    /// # use time::Time;
    /// Time::from_hms_nano(0, 0, 0, 1_000_000_000); // 1_000_000_000 isn't a valid nanosecond.
    /// ```
    #[inline(always)]
    #[cfg(feature = "panicking-api")]
    #[cfg_attr(doc, doc(cfg(feature = "panicking-api")))]
    #[deprecated(
        since = "0.2.3",
        note = "For times knowable at compile-time, use the `time!` macro. For situations where a \
                value isn't known, use `Time::try_from_hms_nano`."
    )]
    pub fn from_hms_nano(hour: u8, minute: u8, second: u8, nanosecond: u32) -> Self {
        assert_value_in_range!(hour in 0 => 23);
        assert_value_in_range!(minute in 0 => 59);
        assert_value_in_range!(second in 0 => 59);
        assert_value_in_range!(nanosecond in 0 => 999_999_999);
        Self {
            hour,
            minute,
            second,
            nanosecond,
        }
    }

    /// Attempt to create a `Time` from the hour, minute, second, and nanosecond.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::try_from_hms_nano(1, 2, 3, 4).is_ok());
    /// ```
    ///
    /// Returns `None` if any component is not valid.
    ///
    /// ```rust
    /// # use time::Time;
    /// assert!(Time::try_from_hms_nano(24, 0, 0, 0).is_err()); // 24 isn't a valid hour.
    /// assert!(Time::try_from_hms_nano(0, 60, 0, 0).is_err()); // 60 isn't a valid minute.
    /// assert!(Time::try_from_hms_nano(0, 0, 60, 0).is_err()); // 60 isn't a valid second.
    /// assert!(Time::try_from_hms_nano(0, 0, 0, 1_000_000_000).is_err()); // 1_000_000_000 isn't a valid nanosecond.
    /// ```
    #[inline(always)]
    pub fn try_from_hms_nano(
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Result<Self, ComponentRangeError> {
        ensure_value_in_range!(hour in 0 => 23);
        ensure_value_in_range!(minute in 0 => 59);
        ensure_value_in_range!(second in 0 => 59);
        ensure_value_in_range!(nanosecond in 0 => 999_999_999);
        Ok(Self {
            hour,
            minute,
            second,
            nanosecond,
        })
    }

    /// Create a `Time` representing the current time (UTC).
    ///
    /// ```rust,no_run
    /// # use time::Time;
    /// println!("{:?}", Time::now());
    /// ```
    #[inline(always)]
    #[cfg(feature = "std")]
    #[cfg_attr(doc, doc(cfg(feature = "std")))]
    pub fn now() -> Self {
        PrimitiveDateTime::now().time()
    }

    /// Get the clock hour.
    ///
    /// The returned value will always be in the range `0..24`.
    ///
    /// ```rust
    /// # use time::time;
    /// assert_eq!(time!(0:00:00).hour(), 0);
    /// assert_eq!(time!(23:59:59).hour(), 23);
    /// ```
    #[inline(always)]
    pub const fn hour(self) -> u8 {
        self.hour
    }

    /// Get the minute within the hour.
    ///
    /// The returned value will always be in the range `0..60`.
    ///
    /// ```rust
    /// # use time::time;
    /// assert_eq!(time!(0:00:00).minute(), 0);
    /// assert_eq!(time!(23:59:59).minute(), 59);
    /// ```
    #[inline(always)]
    pub const fn minute(self) -> u8 {
        self.minute
    }

    /// Get the second within the minute.
    ///
    /// The returned value will always be in the range `0..60`.
    ///
    /// ```rust
    /// # use time::time;
    /// assert_eq!(time!(0:00:00).second(), 0);
    /// assert_eq!(time!(23:59:59).second(), 59);
    /// ```
    #[inline(always)]
    pub const fn second(self) -> u8 {
        self.second
    }

    /// Get the milliseconds within the second.
    ///
    /// The returned value will always be in the range `0..1_000`.
    ///
    /// ```rust
    /// # use time::time;
    /// assert_eq!(time!(0:00).millisecond(), 0);
    /// assert_eq!(time!(23:59:59.999).millisecond(), 999);
    /// ```
    #[inline(always)]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn millisecond(self) -> u16 {
        (self.nanosecond() / 1_000_000) as u16
    }

    /// Get the microseconds within the second.
    ///
    /// The returned value will always be in the range `0..1_000_000`.
    ///
    /// ```rust
    /// # use time::time;
    /// assert_eq!(time!(0:00).microsecond(), 0);
    /// assert_eq!(time!(23:59:59.999_999).microsecond(), 999_999);
    /// ```
    #[inline(always)]
    pub const fn microsecond(self) -> u32 {
        self.nanosecond() / 1_000
    }

    /// Get the nanoseconds within the second.
    ///
    /// The returned value will always be in the range `0..1_000_000_000`.
    ///
    /// ```rust
    /// # use time::time;
    /// assert_eq!(time!(0:00).nanosecond(), 0);
    /// assert_eq!(time!(23:59:59.999_999_999).nanosecond(), 999_999_999);
    /// ```
    #[inline(always)]
    pub const fn nanosecond(self) -> u32 {
        self.nanosecond
    }

    /// Get the number of nanoseconds since midnight.
    #[inline(always)]
    pub(crate) const fn nanoseconds_since_midnight(self) -> u64 {
        self.hour() as u64 * 60 * 60 * 1_000_000_000
            + self.minute() as u64 * 60 * 1_000_000_000
            + self.second() as u64 * 1_000_000_000
            + self.nanosecond() as u64
    }

    /// Create a `Time` from the number of nanoseconds since midnight.
    #[inline(always)]
    pub(crate) const fn from_nanoseconds_since_midnight(mut nanosecond: u64) -> Self {
        #![allow(clippy::cast_possible_truncation)]

        nanosecond %= 86_400 * 1_000_000_000;

        Self {
            hour: (nanosecond / 1_000_000_000 / 60 / 60) as u8,
            minute: (nanosecond / 1_000_000_000 / 60 % 60) as u8,
            second: (nanosecond / 1_000_000_000 % 60) as u8,
            nanosecond: (nanosecond % 1_000_000_000) as u32,
        }
    }
}

/// Methods that allow formatting the `Time`.
impl Time {
    /// Format the `Time` using the provided string.
    ///
    /// ```rust
    /// # use time::time;
    /// assert_eq!(time!(0:00).format("%r"), "12:00:00 am");
    /// ```
    #[inline(always)]
    pub fn format(self, format: &str) -> String {
        DeferredFormat {
            date: None,
            time: Some(self),
            offset: None,
            format: crate::format::parse_fmt_string(format),
        }
        .to_string()
    }

    /// Attempt to parse a `Time` using the provided string.
    ///
    /// ```rust
    /// # use time::{Time, time};
    /// assert_eq!(
    ///     Time::parse("0:00:00", "%T"),
    ///     Ok(time!(0:00))
    /// );
    /// assert_eq!(
    ///     Time::parse("23:59:59", "%T"),
    ///     Ok(time!(23:59:59))
    /// );
    /// assert_eq!(
    ///     Time::parse("12:00:00 am", "%r"),
    ///     Ok(time!(0:00))
    /// );
    /// assert_eq!(
    ///     Time::parse("12:00:00 pm", "%r"),
    ///     Ok(time!(12:00))
    /// );
    /// assert_eq!(
    ///     Time::parse("11:59:59 pm", "%r"),
    ///     Ok(time!(23:59:59))
    /// );
    /// ```
    #[inline(always)]
    pub fn parse(s: &str, format: &str) -> ParseResult<Self> {
        Self::try_from_parsed_items(parse(s, format)?)
    }

    /// Given the items already parsed, attempt to create a `Time`.
    #[inline]
    pub(crate) fn try_from_parsed_items(items: ParsedItems) -> ParseResult<Self> {
        macro_rules! items {
            ($($item:ident),* $(,)?) => {
                ParsedItems { $($item: Some($item)),*, .. }
            };
        }

        /// Convert a 12-hour time to a 24-hour time.
        #[inline(always)]
        fn hour_12_to_24(hour: NonZeroU8, am_pm: AmPm) -> u8 {
            use AmPm::{AM, PM};
            match (hour.get(), am_pm) {
                (12, AM) => 0,
                (12, PM) => 12,
                (h, AM) => h,
                (h, PM) => h + 12,
            }
        }

        match items {
            items!(hour_24, minute, second) => Ok(Self::try_from_hms(hour_24, minute, second)
                .expect("components are checked when parsing")),
            items!(hour_12, minute, second, am_pm) => {
                Ok(
                    Self::try_from_hms(hour_12_to_24(hour_12, am_pm), minute, second)
                        .expect("components are checked when parsing"),
                )
            }
            items!(hour_24, minute) => Ok(Self::try_from_hms(hour_24, minute, 0)
                .expect("components are checked when parsing")),
            items!(hour_12, minute, am_pm) => {
                Ok(Self::try_from_hms(hour_12_to_24(hour_12, am_pm), minute, 0)
                    .expect("components are checked when parsing"))
            }
            items!(hour_24) => {
                Ok(Self::try_from_hms(hour_24, 0, 0).expect("components are checked when parsing"))
            }
            items!(hour_12, am_pm) => Ok(Self::try_from_hms(hour_12_to_24(hour_12, am_pm), 0, 0)
                .expect("components are checked when parsing")),
            _ => Err(ParseError::InsufficientInformation),
        }
    }
}

impl Add<Duration> for Time {
    type Output = Self;

    /// Add the sub-day time of the `Duration` to the `Time`. Wraps on overflow.
    ///
    /// ```rust
    /// # use time::{Duration, time};
    /// assert_eq!(
    ///     time!(12:00) + Duration::hours(2),
    ///     time!(14:00)
    /// );
    /// assert_eq!(
    ///     time!(0:00:01) + Duration::seconds(-2),
    ///     time!(23:59:59)
    /// );
    /// ```
    #[inline(always)]
    fn add(self, duration: Duration) -> Self::Output {
        // TODO cast_sign_loss: rust-lang/rust-clippy#4818
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Self::from_nanoseconds_since_midnight(
            self.nanoseconds_since_midnight()
                + duration
                    .whole_nanoseconds()
                    .rem_euclid(NANOS_PER_DAY as i128) as u64,
        )
    }
}

impl Add<StdDuration> for Time {
    type Output = Self;

    /// Add the sub-day time of the `std::time::Duration` to the `Time`. Wraps
    /// on overflow.
    ///
    /// ```rust
    /// # use time::time;
    /// # use core::time::Duration;
    /// assert_eq!(
    ///     time!(12:00) + Duration::from_secs(2 * 3_600),
    ///     time!(14:00)
    /// );
    /// assert_eq!(
    ///     time!(23:59:59) + Duration::from_secs(2),
    ///     time!(0:00:01)
    /// );
    /// ```
    #[inline(always)]
    fn add(self, duration: StdDuration) -> Self::Output {
        self + Duration::seconds((duration.as_secs() % 86_400) as i64)
    }
}

impl AddAssign<Duration> for Time {
    /// Add the sub-day time of the `Duration` to the existing `Time`. Wraps on
    /// overflow.
    ///
    /// ```rust
    /// # use time::{Duration, time};
    /// let mut time = time!(12:00);
    /// time += Duration::hours(2);
    /// assert_eq!(time, time!(14:00));
    ///
    /// let mut time = time!(0:00:01);
    /// time += Duration::seconds(-2);
    /// assert_eq!(time, time!(23:59:59));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, duration: Duration) {
        *self = *self + duration;
    }
}

impl AddAssign<StdDuration> for Time {
    /// Add the sub-day time of the `std::time::Duration` to the existing
    /// `Time`. Wraps on overflow.
    ///
    /// ```rust
    /// # use time::time;
    /// # use core::time::Duration;
    /// let mut time = time!(12:00);
    /// time += Duration::from_secs(2 * 3_600);
    /// assert_eq!(time, time!(14:00));
    ///
    /// let mut time = time!(23:59:59);
    /// time += Duration::from_secs(2);
    /// assert_eq!(time, time!(0:00:01));
    /// ```
    #[inline(always)]
    fn add_assign(&mut self, duration: StdDuration) {
        *self = *self + duration;
    }
}

impl Sub<Duration> for Time {
    type Output = Self;

    /// Subtract the sub-day time of the `Duration` from the `Time`. Wraps on
    /// overflow.
    ///
    /// ```rust
    /// # use time::{Duration, time};
    /// assert_eq!(
    ///     time!(14:00) - Duration::hours(2),
    ///     time!(12:00)
    /// );
    /// assert_eq!(
    ///     time!(23:59:59) - Duration::seconds(-2),
    ///     time!(0:00:01)
    /// );
    /// ```
    #[inline(always)]
    fn sub(self, duration: Duration) -> Self::Output {
        self + -duration
    }
}

impl Sub<StdDuration> for Time {
    type Output = Self;

    /// Subtract the sub-day time of the `std::time::Duration` from the `Time`.
    /// Wraps on overflow.
    ///
    /// ```rust
    /// # use time::time;
    /// # use core::time::Duration;
    /// assert_eq!(
    ///     time!(14:00) - Duration::from_secs(2 * 3_600),
    ///     time!(12:00)
    /// );
    /// assert_eq!(
    ///     time!(0:00:01) - Duration::from_secs(2),
    ///     time!(23:59:59)
    /// );
    /// ```
    #[inline(always)]
    fn sub(self, duration: StdDuration) -> Self::Output {
        self - Duration::seconds((duration.as_secs() % 86_400) as i64)
    }
}

impl SubAssign<Duration> for Time {
    /// Subtract the sub-day time of the `Duration` from the existing `Time`.
    /// Wraps on overflow.
    ///
    /// ```rust
    /// # use time::{Duration, time};
    /// let mut time = time!(14:00);
    /// time -= Duration::hours(2);
    /// assert_eq!(time, time!(12:00));
    ///
    /// let mut time = time!(23:59:59);
    /// time -= Duration::seconds(-2);
    /// assert_eq!(time, time!(0:00:01));
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, duration: Duration) {
        *self = *self - duration;
    }
}

impl SubAssign<StdDuration> for Time {
    /// Subtract the sub-day time of the `std::time::Duration` from the existing
    /// `Time`. Wraps on overflow.
    ///
    /// ```rust
    /// # use time::time;
    /// # use core::time::Duration;
    /// let mut time = time!(14:00);
    /// time -= Duration::from_secs(2 * 3_600);
    /// assert_eq!(time, time!(12:00));
    ///
    /// let mut time = time!(0:00:01);
    /// time -= Duration::from_secs(2);
    /// assert_eq!(time, time!(23:59:59));
    /// ```
    #[inline(always)]
    fn sub_assign(&mut self, duration: StdDuration) {
        *self = *self - duration;
    }
}

impl Sub<Time> for Time {
    type Output = Duration;

    /// Subtract two `Time`s, returning the `Duration` between. This assumes
    /// both `Time`s are in the same calendar day.
    ///
    /// ```rust
    /// use time::{Duration, time};
    /// assert_eq!(
    ///     time!(0:00) - time!(0:00),
    ///     Duration::zero()
    /// );
    /// assert_eq!(
    ///     time!(1:00) - time!(0:00),
    ///     Duration::hour()
    /// );
    /// assert_eq!(
    ///     time!(0:00) - time!(1:00),
    ///     Duration::hours(-1)
    /// );
    /// assert_eq!(
    ///     time!(0:00) - time!(23:00),
    ///     Duration::hours(-23)
    /// );
    /// ```
    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Duration::nanoseconds(
            self.nanoseconds_since_midnight() as i64 - rhs.nanoseconds_since_midnight() as i64,
        )
    }
}

impl PartialOrd for Time {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.cmp(other).into()
    }
}

impl Ord for Time {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        use Ordering::*;

        match self.hour().cmp(&other.hour()) {
            Less => Less,
            Greater => Greater,
            Equal => match self.minute().cmp(&other.minute()) {
                Less => Less,
                Greater => Greater,
                Equal => match self.second().cmp(&other.second()) {
                    Less => Less,
                    Greater => Greater,
                    Equal => self.nanosecond().cmp(&other.nanosecond()),
                },
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::result_unwrap_used)]
mod test {
    use super::*;
    use crate::prelude::*;

    #[test]
    fn nanoseconds_since_midnight() {
        let time = time!(0:00);
        assert_eq!(time.nanoseconds_since_midnight(), 0);
        assert_eq!(Time::from_nanoseconds_since_midnight(0), time);

        let time = time!(23:59:59.999_999_999);
        assert_eq!(time.nanoseconds_since_midnight(), NANOS_PER_DAY - 1);
        assert_eq!(
            Time::from_nanoseconds_since_midnight(NANOS_PER_DAY - 1),
            time
        );
    }

    #[test]
    fn midnight() {
        assert_eq!(Time::midnight(), time!(0:00));
    }

    #[test]
    #[cfg(feature = "panicking-api")]
    #[allow(deprecated)]
    fn from_hms() {
        let time = Time::from_hms(1, 2, 3);
        assert_eq!(time.hour(), 1);
        assert_eq!(time.minute(), 2);
        assert_eq!(time.second(), 3);
        assert_eq!(time.nanosecond(), 0);

        #[cfg(feature = "std")]
        {
            assert_panics!(Time::from_hms(24, 0, 0), "24 isn't a valid hour");
            assert_panics!(Time::from_hms(0, 60, 0), "60 isn't a valid minute");
            assert_panics!(Time::from_hms(0, 0, 60), "60 isn't a valid second");
        }
    }

    #[test]
    fn try_from_hms() {
        let time = Time::try_from_hms(1, 2, 3).unwrap();
        assert_eq!(time.hour(), 1);
        assert_eq!(time.minute(), 2);
        assert_eq!(time.second(), 3);
        assert_eq!(time.nanosecond(), 0);

        assert!(Time::try_from_hms(24, 0, 0).is_err());
        assert!(Time::try_from_hms(0, 60, 0).is_err());
        assert!(Time::try_from_hms(0, 0, 60).is_err());
    }

    #[test]
    #[cfg(feature = "panicking-api")]
    #[allow(deprecated)]
    fn from_hms_milli() {
        let time = Time::from_hms_milli(1, 2, 3, 4);
        assert_eq!(time.hour(), 1);
        assert_eq!(time.minute(), 2);
        assert_eq!(time.second(), 3);
        assert_eq!(time.millisecond(), 4);
        assert_eq!(time.nanosecond(), 4_000_000);

        #[cfg(feature = "std")]
        {
            assert_panics!(Time::from_hms_milli(24, 0, 0, 0), "24 isn't a valid hour");
            assert_panics!(Time::from_hms_milli(0, 60, 0, 0), "60 isn't a valid minute");
            assert_panics!(Time::from_hms_milli(0, 0, 60, 0), "60 isn't a valid second");
            assert_panics!(
                Time::from_hms_milli(0, 0, 0, 1_000),
                "1_000 isn't a valid millisecond"
            );
        }
    }

    #[test]
    fn try_from_hms_milli() {
        let time = Time::try_from_hms_milli(1, 2, 3, 4).unwrap();
        assert_eq!(time.hour(), 1);
        assert_eq!(time.minute(), 2);
        assert_eq!(time.second(), 3);
        assert_eq!(time.millisecond(), 4);
        assert_eq!(time.nanosecond(), 4_000_000);

        assert!(Time::try_from_hms_milli(24, 0, 0, 0).is_err());
        assert!(Time::try_from_hms_milli(0, 60, 0, 0).is_err());
        assert!(Time::try_from_hms_milli(0, 0, 60, 0).is_err());
        assert!(Time::try_from_hms_milli(0, 0, 0, 1_000).is_err());
    }

    #[test]
    #[cfg(feature = "panicking-api")]
    #[allow(deprecated)]
    fn from_hms_micro() {
        let time = Time::from_hms_micro(1, 2, 3, 4);
        assert_eq!(time.hour(), 1);
        assert_eq!(time.minute(), 2);
        assert_eq!(time.second(), 3);
        assert_eq!(time.microsecond(), 4);
        assert_eq!(time.nanosecond(), 4_000);

        #[cfg(feature = "std")]
        {
            assert_panics!(Time::from_hms_micro(24, 0, 0, 0), "24 isn't a valid hour");
            assert_panics!(Time::from_hms_micro(0, 60, 0, 0), "60 isn't a valid minute");
            assert_panics!(Time::from_hms_micro(0, 0, 60, 0), "60 isn't a valid second");
            assert_panics!(
                Time::from_hms_micro(0, 0, 0, 1_000_000),
                "1_000_000 isn't a valid microsecond"
            );
        }
    }

    #[test]
    fn try_from_hms_micro() {
        let time = Time::try_from_hms_micro(1, 2, 3, 4).unwrap();
        assert_eq!(time.hour(), 1);
        assert_eq!(time.minute(), 2);
        assert_eq!(time.second(), 3);
        assert_eq!(time.microsecond(), 4);
        assert_eq!(time.nanosecond(), 4_000);

        assert!(Time::try_from_hms_micro(24, 0, 0, 0).is_err());
        assert!(Time::try_from_hms_micro(0, 60, 0, 0).is_err());
        assert!(Time::try_from_hms_micro(0, 0, 60, 0).is_err());
        assert!(Time::try_from_hms_micro(0, 0, 0, 1_000_000).is_err());
    }

    #[test]
    #[cfg(feature = "panicking-api")]
    #[allow(deprecated)]
    fn from_hms_nano() {
        let time = Time::from_hms_nano(1, 2, 3, 4);
        assert_eq!(time.hour(), 1);
        assert_eq!(time.minute(), 2);
        assert_eq!(time.second(), 3);
        assert_eq!(time.nanosecond(), 4);

        #[cfg(feature = "std")]
        {
            assert_panics!(Time::from_hms_nano(24, 0, 0, 0), "24 isn't a valid hour.");
            assert_panics!(Time::from_hms_nano(0, 60, 0, 0), "60 isn't a valid minute.");
            assert_panics!(Time::from_hms_nano(0, 0, 60, 0), "60 isn't a valid second.");
            assert_panics!(
                Time::from_hms_nano(0, 0, 0, 1_000_000_000),
                "1_000_000_000 isn't a valid nanosecond."
            );
        }
    }

    #[test]
    fn try_from_hms_nano() {
        let time = Time::try_from_hms_nano(1, 2, 3, 4).unwrap();
        assert_eq!(time.hour(), 1);
        assert_eq!(time.minute(), 2);
        assert_eq!(time.second(), 3);
        assert_eq!(time.nanosecond(), 4);

        assert!(Time::try_from_hms_nano(24, 0, 0, 0).is_err());
        assert!(Time::try_from_hms_nano(0, 60, 0, 0).is_err());
        assert!(Time::try_from_hms_nano(0, 0, 60, 0).is_err());
        assert!(Time::try_from_hms_nano(0, 0, 0, 1_000_000_000).is_err());
    }

    #[test]
    fn hour() {
        for hour in 0..24 {
            assert_eq!(Time::try_from_hms(hour, 0, 0).unwrap().hour(), hour);
            assert_eq!(Time::try_from_hms(hour, 59, 59).unwrap().hour(), hour);
        }
    }

    #[test]
    fn minute() {
        for minute in 0..60 {
            assert_eq!(Time::try_from_hms(0, minute, 0).unwrap().minute(), minute);
            assert_eq!(Time::try_from_hms(23, minute, 59).unwrap().minute(), minute);
        }
    }

    #[test]
    fn second() {
        for second in 0..60 {
            assert_eq!(Time::try_from_hms(0, 0, second).unwrap().second(), second);
            assert_eq!(Time::try_from_hms(23, 59, second).unwrap().second(), second);
        }
    }

    #[test]
    fn millisecond() {
        for milli in 0..1_000 {
            assert_eq!(
                Time::try_from_hms_milli(0, 0, 0, milli)
                    .unwrap()
                    .millisecond(),
                milli
            );
            assert_eq!(
                Time::try_from_hms_milli(23, 59, 59, milli)
                    .unwrap()
                    .millisecond(),
                milli
            );
        }
    }

    #[test]
    fn microsecond() {
        for micro in (0..1_000_000).step_by(1_000) {
            assert_eq!(
                Time::try_from_hms_micro(0, 0, 0, micro)
                    .unwrap()
                    .microsecond(),
                micro
            );
            assert_eq!(
                Time::try_from_hms_micro(23, 59, 59, micro)
                    .unwrap()
                    .microsecond(),
                micro
            );
        }
    }

    #[test]
    fn nanosecond() {
        for nano in (0..1_000_000_000).step_by(1_000_000) {
            assert_eq!(
                Time::try_from_hms_nano(0, 0, 0, nano).unwrap().nanosecond(),
                nano
            );
            assert_eq!(
                Time::try_from_hms_nano(23, 59, 59, nano)
                    .unwrap()
                    .nanosecond(),
                nano
            );
        }
    }

    #[test]
    fn format() {
        assert_eq!(time!(0:00).format("%T"), "0:00:00");
        assert_eq!(time!(12:00 am).format("%r"), "12:00:00 am");
        assert_eq!(time!(23:59:59).format("%T"), "23:59:59");
        assert_eq!(time!(11:59:59 pm).format("%r"), "11:59:59 pm");
    }

    #[test]
    fn parse() {
        assert_eq!(Time::parse("0:00:00", "%T"), Ok(time!(0:00)));
        assert_eq!(Time::parse("23:59:59", "%T"), Ok(time!(23:59:59)));
        assert_eq!(Time::parse("1:00:00 am", "%r"), Ok(time!(1:00)));
        assert_eq!(Time::parse("12:00:00 am", "%r"), Ok(time!(12:00 am)));
        assert_eq!(Time::parse("12:00:00 pm", "%r"), Ok(time!(12:00 pm)));
        assert_eq!(Time::parse("11:59:59 pm", "%r"), Ok(time!(11:59:59 pm)));
    }

    #[test]
    fn parse_missing_seconds() {
        // Missing seconds defaults to zero.
        assert_eq!(Time::parse("0:00", "%-H:%M"), Ok(time!(0:00)));
        assert_eq!(Time::parse("23:59", "%H:%M"), Ok(time!(23:59)));
        assert_eq!(Time::parse("12:00 am", "%I:%M %p"), Ok(time!(12:00 am)));
        assert_eq!(Time::parse("12:00 pm", "%I:%M %p"), Ok(time!(12:00 pm)));
    }

    #[test]
    fn parse_missing_minutes() {
        // Missing minutes defaults to zero.
        assert_eq!(Time::parse("0", "%-H"), Ok(time!(0:00)));
        assert_eq!(Time::parse("23", "%H"), Ok(time!(23:00)));
        assert_eq!(Time::parse("12am", "%I%p"), Ok(time!(12:00 am)));
        assert_eq!(Time::parse("12pm", "%I%p"), Ok(time!(12:00 pm)));
    }

    #[test]
    fn add_duration() {
        assert_eq!(time!(0:00) + 1.seconds(), time!(0:00:01));
        assert_eq!(time!(0:00) + 1.minutes(), time!(0:01));
        assert_eq!(time!(0:00) + 1.hours(), time!(1:00));
        assert_eq!(time!(0:00) + 1.days(), time!(0:00));
    }

    #[test]
    fn add_assign_duration() {
        let mut time = time!(0:00);

        time += 1.seconds();
        assert_eq!(time, time!(0:00:01));

        time += 1.minutes();
        assert_eq!(time, time!(0:01:01));

        time += 1.hours();
        assert_eq!(time, time!(1:01:01));

        time += 1.days();
        assert_eq!(time, time!(1:01:01));
    }

    #[test]
    fn sub_duration() {
        assert_eq!(time!(12:00) - 1.hours(), time!(11:00));

        // Underflow
        assert_eq!(time!(0:00) - 1.seconds(), time!(23:59:59));
        assert_eq!(time!(0:00) - 1.minutes(), time!(23:59));
        assert_eq!(time!(0:00) - 1.hours(), time!(23:00));
        assert_eq!(time!(0:00) - 1.days(), time!(0:00));
    }

    #[test]
    fn sub_assign_duration() {
        let mut time = time!(0:00);

        time -= 1.seconds();
        assert_eq!(time, time!(23:59:59));

        time -= 1.minutes();
        assert_eq!(time, time!(23:58:59));

        time -= 1.hours();
        assert_eq!(time, time!(22:58:59));

        time -= 1.days();
        assert_eq!(time, time!(22:58:59));
    }

    #[test]
    fn add_std_duration() {
        assert_eq!(time!(0:00) + 1.std_seconds(), time!(0:00:01));
        assert_eq!(time!(0:00) + 1.std_minutes(), time!(0:01));
        assert_eq!(time!(0:00) + 1.std_hours(), time!(1:00));
        assert_eq!(time!(0:00) + 1.std_days(), time!(0:00));
    }

    #[test]
    fn add_assign_std_duration() {
        let mut time = time!(0:00);

        time += 1.std_seconds();
        assert_eq!(time, time!(0:00:01));

        time += 1.std_minutes();
        assert_eq!(time, time!(0:01:01));

        time += 1.std_hours();
        assert_eq!(time, time!(1:01:01));

        time += 1.std_days();
        assert_eq!(time, time!(1:01:01));
    }

    #[test]
    fn sub_std_duration() {
        assert_eq!(time!(12:00) - 1.std_hours(), time!(11:00));

        // Underflow
        assert_eq!(time!(0:00) - 1.std_seconds(), time!(23:59:59));
        assert_eq!(time!(0:00) - 1.std_minutes(), time!(23:59));
        assert_eq!(time!(0:00) - 1.std_hours(), time!(23:00));
        assert_eq!(time!(0:00) - 1.std_days(), time!(0:00));
    }

    #[test]
    fn sub_assign_std_duration() {
        let mut time = time!(0:00);

        time -= 1.std_seconds();
        assert_eq!(time, time!(23:59:59));

        time -= 1.std_minutes();
        assert_eq!(time, time!(23:58:59));

        time -= 1.std_hours();
        assert_eq!(time, time!(22:58:59));

        time -= 1.std_days();
        assert_eq!(time, time!(22:58:59));
    }

    #[test]
    fn sub_time() {
        assert_eq!(time!(0:00) - time!(0:00), 0.seconds());
        assert_eq!(time!(1:00) - time!(0:00), 1.hours());
        assert_eq!(time!(1:00) - time!(0:00:01), 59.minutes() + 59.seconds());
    }

    #[test]
    fn ordering() {
        assert!(time!(0:00) < time!(0:00:00.000_000_001));
        assert!(time!(0:00) < time!(0:00:01));
        assert!(time!(12:00) > time!(11:00));
        assert_eq!(time!(0:00), time!(0:00));
    }
}
