#[macro_export]
macro_rules! create_date_from_map {
    ($fn_name:ident, $field_name:literal) => {
        fn $fn_name(
            map: &mut HashMap<String, AttributeValue>,
        ) -> Result<DateTime<Utc>, ConversionError> {
            match map
                .remove($field_name)
                .ok_or(ConversionError::MissingField($field_name))?
            {
                AttributeValue::N(string) => {
                    let seconds: i64 = string.parse().map_err(|_| {
                        ConversionError::UnexpectedDataType(
                            concat!($field_name, ". could not convert the string into an int")
                        )
                    })?;
                    DateTime::from_timestamp(seconds, 0).ok_or(ConversionError::UnexpectedDataType(
                        concat!($field_name, ". could not convert the int into a DateTime")
                    ))
                }
                _ => Err(ConversionError::UnexpectedDataType($field_name)),
            }
        }
    };
}
