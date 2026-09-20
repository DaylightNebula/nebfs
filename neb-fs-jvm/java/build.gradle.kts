plugins {
    `java-library`
    `maven-publish`
}

group = "io.github.daylightnebula"
version = "0.1.0"

// Sources live directly under src/ (no main/java split) — point Gradle at
// the existing layout instead of moving files to match its convention.
sourceSets {
    main {
        java.setSrcDirs(listOf("src"))
    }
}

tasks.withType<JavaCompile> {
    // FFM API is stable from 22 (JEP 454); building/running needs a JDK 22+
    // toolchain, but no --enable-preview.
    options.release.set(22)
}

val cargoBuildNative = tasks.register<Exec>("cargoBuildNative") {
    workingDir = file("..")
    commandLine("cargo", "build", "--release")
    outputs.file("../target/release/${System.mapLibraryName("neb_fs_jvm")}")
}

val copyNativeLib = tasks.register<Copy>("copyNativeLib") {
    dependsOn(cargoBuildNative)
    from("../target/release/${System.mapLibraryName("neb_fs_jvm")}")
    into(layout.buildDirectory.dir("native-resources/native"))
}

sourceSets {
    main {
        resources.srcDir(layout.buildDirectory.dir("native-resources"))
    }
}

tasks.named("processResources") {
    dependsOn(copyNativeLib)
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])
        }
    }
}
