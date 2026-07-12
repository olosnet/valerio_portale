use validator::ValidationError;

pub fn parse_seconds_component(value: &str) -> Option<f64> {
    let (whole_part, fractional_part) = match value.split_once('.') {
        Some((whole_part, fractional_part)) => (whole_part, Some(fractional_part)),
        None => (value, None),
    };

    if whole_part.is_empty() || whole_part.len() > 2 {
        return None;
    }

    if !whole_part.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }

    if let Some(fractional_part) = fractional_part {
        if fractional_part.is_empty() || !fractional_part.chars().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
    }

    value.parse::<f64>().ok().filter(|seconds| *seconds >= 0.0)
}

pub fn validate_coord_ar(value: &String) -> Result<(), ValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(());
    }

    let is_valid = (1..=2).any(|hour_len| {
        (1..=2).any(|minute_len| {
            if value.len() <= hour_len + minute_len {
                return false;
            }

            let (hour_part, remainder) = value.split_at(hour_len);
            let (minute_part, second_part) = remainder.split_at(minute_len);

            let hours = hour_part.parse::<u8>().ok().filter(|hours| *hours < 24);
            let minutes = minute_part
                .parse::<u8>()
                .ok()
                .filter(|minutes| *minutes < 60);
            let seconds = parse_seconds_component(second_part).filter(|seconds| *seconds < 60.0);

            hours.is_some() && minutes.is_some() && seconds.is_some()
        })
    });

    if is_valid {
        Ok(())
    } else {
        let mut error = ValidationError::new("coord_ar");
        error.message = Some("Invalid right ascension format".into());
        Err(error)
    }
}

pub fn validate_coord_dec(value: &String) -> Result<(), ValidationError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(());
    }

    let Some(sign) = value.chars().next() else {
        let mut error = ValidationError::new("coord_dec");
        error.message = Some("Invalid declination format".into());
        return Err(error);
    };

    if sign != '+' && sign != '-' {
        let mut error = ValidationError::new("coord_dec");
        error.message = Some("Invalid declination format".into());
        return Err(error);
    }

    let remainder = &value[1..];
    let is_valid = (1..=2).any(|degree_len| {
        (1..=2).any(|minute_len| {
            if remainder.len() <= degree_len + minute_len {
                return false;
            }

            let (degree_part, remainder) = remainder.split_at(degree_len);
            let (minute_part, second_part) = remainder.split_at(minute_len);

            if second_part.is_empty() || second_part.len() > 2 {
                return false;
            }

            let degrees = degree_part
                .parse::<u8>()
                .ok()
                .filter(|degrees| *degrees <= 90);
            let minutes = minute_part
                .parse::<u8>()
                .ok()
                .filter(|minutes| *minutes < 60);
            let seconds = second_part
                .parse::<u8>()
                .ok()
                .filter(|seconds| *seconds < 60);

            degrees.is_some() && minutes.is_some() && seconds.is_some()
        })
    });

    if is_valid {
        Ok(())
    } else {
        let mut error = ValidationError::new("coord_dec");
        error.message = Some("Invalid declination format".into());
        Err(error)
    }
}
