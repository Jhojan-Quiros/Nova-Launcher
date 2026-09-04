use nova_launcher_lib::infrastructure::java::detector::JavaVersionParser;

#[test]
fn test_java_version_parser() {
    // 1. Modern Java 25
    let out_java25 = r#"java version "25.0.2" 2026-01-20 LTS
Java(TM) SE Runtime Environment (build 25.0.2+10-LTS-69)
Java HotSpot(TM) 64-Bit Server VM (build 25.0.2+10-LTS-69, mixed mode, sharing)"#;
    let res25 = JavaVersionParser::parse_version(out_java25).expect("Should parse Java 25");
    assert_eq!(res25.0, 25);
    assert_eq!(res25.1, "25.0.2");

    // 2. OpenJDK 21
    let out_java21 = r#"openjdk version "21.0.2" 2024-01-16
OpenJDK Runtime Environment Temurin-21.0.2+13 (build 21.0.2+13)
OpenJDK 64-Bit Server VM Temurin-21.0.2+13 (build 21.0.2+13, mixed mode, sharing)"#;
    let res21 = JavaVersionParser::parse_version(out_java21).expect("Should parse OpenJDK 21");
    assert_eq!(res21.0, 21);
    assert_eq!(res21.1, "21.0.2");

    // 3. OpenJDK 17
    let out_java17 = r#"openjdk version "17.0.9" 2023-10-17
OpenJDK Runtime Environment (build 17.0.9+9)
OpenJDK 64-Bit Server VM (build 17.0.9+9, mixed mode)"#;
    let res17 = JavaVersionParser::parse_version(out_java17).expect("Should parse OpenJDK 17");
    assert_eq!(res17.0, 17);
    assert_eq!(res17.1, "17.0.9");

    // 4. Legacy Java 1.8
    let out_java8 = r#"java version "1.8.0_391"
Java(TM) SE Runtime Environment (build 1.8.0_391-b13)
Java HotSpot(TM) 64-Bit Server VM (build 25.391-b13, mixed mode)"#;
    let res8 = JavaVersionParser::parse_version(out_java8).expect("Should parse Java 1.8");
    assert_eq!(res8.0, 8);
    assert_eq!(res8.1, "1.8.0_391");
}