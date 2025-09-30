use versions::Versioning;

pub fn get_major_version(version: &str) -> Option<String> {
    let versioning = Versioning::new(version);
    match versioning {
        Some(parsed_ver) => match parsed_ver {
            Versioning::Ideal(sem_ver) => {
                if sem_ver.major == 0 {
                    Some(String::from(format!("{}.{}", sem_ver.major, sem_ver.minor)))
                } else {
                    Some(String::from(format!("{}", sem_ver.major)))
                }
            }
            Versioning::General(version) => Some(String::from(format!(
                "{}",
                version
                    .chunks
                    .0
                    .first()
                    .unwrap()
                    .single_digit_lenient()
                    .unwrap()
            ))),
            Versioning::Complex(_) => Some(String::from("versions")),
        },
        None => None,
    }
}

pub fn is_version_equal(old_version: &str, new_version: &str) -> bool {
    let old_versioning = Versioning::new(old_version);
    if old_versioning.is_none() {
        return false;
    }
    let new_versioning = Versioning::new(new_version);
    if new_versioning.is_none() {
        return false;
    }

    return old_versioning.unwrap().eq(&new_versioning.unwrap());
}
