use std::error::Error;
use std::fmt;
use std::backtrace::Backtrace;
use tracing_error::SpanTrace;

#[derive(Debug)]
pub struct TrasyError<T> {
    context: SpanTrace,
    backtrace: Option<Backtrace>,
    inner: T,
}

impl<T> TrasyError<T> {
    pub fn new(inner: T) -> Self {
        Self {
            context: SpanTrace::capture(),
            backtrace: None,
            inner,
        }
    }

    pub fn with_backtrace(mut self, backtrace: Backtrace) -> Self {
        self.backtrace = Some(backtrace);
        self
    }
}

impl<T: fmt::Debug + fmt::Display> fmt::Display for TrasyError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error: {}\nContext: {}\n", self.inner, self.context)?;
        if let Some(ref backtrace) = self.backtrace {
            write!(f, "Backtrace: {:?}\n", backtrace)?;
        }
        Ok(())
    }
}

impl<T: fmt::Debug + fmt::Display + Error + AsRef<dyn Error>> Error for TrasyError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.inner.as_ref())
    }
}

#[macro_export]
macro_rules! error {
    ($e:expr) => {
        TrasyError::new($e).with_backtrace(std::backtrace::Backtrace::capture())
    };
}

#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        Err(TrasyError::new($e).with_backtrace(std::backtrace::Backtrace::capture()))
    };
}
