use super::{DbConfig, DbType, percent_decode, percent_encode};

fn tcp_config(host: &str, port: u16) -> DbConfig {
    DbConfig {
        db_type: DbType::Postgres,
        db_host: host.to_owned(),
        db_port: Some(port),
        db_username: "u".to_owned(),
        db_password: "p".to_owned(),
        db_name: "db".to_owned(),
        db_params: Vec::new(),
    }
}

#[test]
fn to_url_encodes_reserved_characters_in_every_component() -> anyhow::Result<()> {
    let config = DbConfig {
        db_type: DbType::Postgres,
        db_host: "db.internal".to_owned(),
        db_port: Some(5433),
        db_username: "app user".to_owned(),
        db_password: "p@ss:w/rd?#%".to_owned(),
        db_name: "be db/x".to_owned(),
        db_params: Vec::new(),
    };
    assert_eq!(
        config.to_url()?,
        "postgres://app%20user:p%40ss%3Aw%2Frd%3F%23%25@db.internal:5433/be%20db%2Fx"
    );
    Ok(())
}

#[test]
fn socket_directory_travels_as_encoded_host_parameter() -> anyhow::Result<()> {
    let mut config = tcp_config("/run/postgresql", 5432);
    config.db_port = None;
    config.db_name = "be_db".to_owned();
    assert_eq!(
        config.to_url()?,
        "postgres://u:p@/be_db?host=%2Frun%2Fpostgresql"
    );
    Ok(())
}

#[test]
fn ipv6_hosts_are_bracketed_instead_of_escaped() -> anyhow::Result<()> {
    assert_eq!(
        tcp_config("::1", 5432).to_url()?,
        "postgres://u:p@[::1]:5432/db"
    );
    let parsed = DbConfig::from_url("postgres://u:p@[2001:db8::5]:6543/db")?;
    assert_eq!(parsed.db_host, "2001:db8::5");
    assert_eq!(parsed.db_port, Some(6543));
    assert_eq!(parsed.to_url()?, "postgres://u:p@[2001:db8::5]:6543/db");
    Ok(())
}

#[test]
fn encoded_url_round_trips_without_double_encoding() -> anyhow::Result<()> {
    let url = "postgres://app%20user:p%40ss%3Aw%2Frd@db.internal:5433/be%20db";
    let parsed = DbConfig::from_url(url)?;
    assert_eq!(parsed.db_username, "app user");
    assert_eq!(parsed.db_password, "p@ss:w/rd");
    assert_eq!(parsed.db_name, "be db");
    assert_eq!(parsed.to_url()?, url);
    Ok(())
}

#[test]
fn libpq_socket_query_is_parsed_and_other_parameters_are_kept() -> anyhow::Result<()> {
    let parsed = DbConfig::from_url(
        "postgres://user:password@/be_db?host=%2Frun%2Fpostgresql&sslmode=disable",
    )?;
    assert_eq!(parsed.db_host, "/run/postgresql");
    assert_eq!(parsed.db_port, None);
    assert_eq!(parsed.db_name, "be_db");
    assert_eq!(
        parsed.to_url()?,
        "postgres://user:password@/be_db?host=%2Frun%2Fpostgresql&sslmode=disable"
    );

    // The documented example names a TCP host through the query parameter.
    let example = DbConfig::from_url("postgres://user:password@/be_db?host=localhost")?;
    assert_eq!(
        example.to_url()?,
        "postgres://user:password@localhost:5432/be_db"
    );
    Ok(())
}

#[test]
fn legacy_unencoded_at_sign_in_password_still_parses() -> anyhow::Result<()> {
    let parsed = DbConfig::from_url("postgres://user:p@ss@localhost/db")?;
    assert_eq!(parsed.db_password, "p@ss");
    assert_eq!(parsed.to_url()?, "postgres://user:p%40ss@localhost:5432/db");
    Ok(())
}

#[test]
fn decoding_rejects_non_utf8_and_keeps_stray_percent_signs() {
    assert!(DbConfig::from_url("postgres://u:%FF@h/db").is_err());
    assert!(DbConfig::from_url("mongodb://u:p@h/db").is_err());
    assert!(DbConfig::from_url("postgres://u:p@h:notaport/db").is_err());
    assert_eq!(percent_decode("100%").ok().as_deref(), Some("100%"));
    assert_eq!(percent_decode("%4").ok().as_deref(), Some("%4"));
    assert_eq!(percent_decode("%zz%41").ok().as_deref(), Some("%zzA"));
    assert_eq!(percent_encode("100%"), "100%25");
    assert_eq!(percent_encode("a-b.c_d~e"), "a-b.c_d~e");
    assert_eq!(percent_encode("한"), "%ED%95%9C");
}
