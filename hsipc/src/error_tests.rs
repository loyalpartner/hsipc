//! Tests for error handling

use crate::Error;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::service_not_found("TestService");
        assert_eq!(err.to_string(), "Service 'TestService' not found");

        let err = Error::method_not_found("TestService", "test_method");
        assert_eq!(
            err.to_string(),
            "Method 'test_method' not found on service 'TestService'"
        );

        let err = Error::timeout("database query", 5000);
        assert_eq!(
            err.to_string(),
            "Operation timed out after 5000ms: database query"
        );
    }

    #[test]
    fn test_error_is_retryable() {
        // Retryable errors
        assert!(Error::transport_msg("network failure").is_retryable());
        assert!(Error::connection_msg("connection lost").is_retryable());
        assert!(Error::timeout("request", 1000).is_retryable());
        assert!(Error::runtime_msg("task failed").is_retryable());

        // Non-retryable errors
        assert!(!Error::service_not_found("TestService").is_retryable());
        assert!(!Error::method_not_found("TestService", "method").is_retryable());
        assert!(!Error::serialization_msg("invalid format").is_retryable());
        assert!(!Error::configuration("invalid config", None).is_retryable());
        assert!(!Error::invalid_topic_pattern("bad/#/pattern").is_retryable());
    }

    #[test]
    fn test_error_category() {
        assert_eq!(Error::transport_msg("error").category(), "transport");
        assert_eq!(
            Error::service_not_found("svc").category(),
            "service_discovery"
        );
        assert_eq!(
            Error::method_not_found("svc", "m").category(),
            "method_resolution"
        );
        assert_eq!(Error::serialization_msg("err").category(), "serialization");
        assert_eq!(Error::connection_msg("err").category(), "connection");
        assert_eq!(Error::timeout("op", 1000).category(), "timeout");
        assert_eq!(Error::runtime_msg("err").category(), "runtime");
        assert_eq!(
            Error::configuration("err", None).category(),
            "configuration"
        );
        assert_eq!(
            Error::invalid_topic_pattern("pat").category(),
            "topic_validation"
        );
    }

    #[test]
    fn test_error_with_source() {
        use std::io;

        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = Error::from(io_err);

        match err {
            Error::Io { message, source } => {
                assert!(message.contains("file not found"));
                assert_eq!(source.kind(), io::ErrorKind::NotFound);
            }
            _ => panic!("Expected Io error"),
        }
    }

    #[test]
    fn test_error_context_fields() {
        let err = Error::configuration("invalid port", Some("server.port".to_string()));
        match err {
            Error::Configuration { message, field } => {
                assert_eq!(message, "invalid port");
                assert_eq!(field, Some("server.port".to_string()));
            }
            _ => panic!("Expected Configuration error"),
        }

        let err = Error::protocol(
            "unexpected message",
            Some("Request".to_string()),
            Some("Event".to_string()),
        );
        match err {
            Error::Protocol {
                message,
                expected,
                received,
            } => {
                assert_eq!(message, "unexpected message");
                assert_eq!(expected, Some("Request".to_string()));
                assert_eq!(received, Some("Event".to_string()));
            }
            _ => panic!("Expected Protocol error"),
        }
    }

    #[test]
    fn test_bincode_error_conversion() {
        // This tests the From<bincode::Error> implementation
        let data = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid data
        let result: Result<String, bincode::Error> = bincode::deserialize(&data);

        if let Err(bincode_err) = result {
            let err: Error = bincode_err.into();
            assert!(matches!(err, Error::Serialization { .. }));
            assert!(err.to_string().contains("Bincode serialization failed"));
        }
    }

    #[test]
    fn test_custom_error_constructors() {
        // Test transport error with source
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let err = Error::transport("connection failed", io_err);
        assert!(matches!(
            err,
            Error::Transport {
                source: Some(_),
                ..
            }
        ));

        // Test runtime error with source
        let runtime_err = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
        let err = Error::runtime("task timeout", runtime_err);
        assert!(matches!(
            err,
            Error::Runtime {
                source: Some(_),
                ..
            }
        ));
    }
}
