pub mod framed;
mod javacli;
mod protocol;

pub mod jvm;

#[cfg(not(any(target_os = "windows", target_os = "redox")))]
const PATH_SEPARATOR: &str = ":";

#[cfg(any(target_os = "windows", target_os = "redox"))]
const PATH_SEPARATOR: &str = ";";

#[cfg(test)]
mod test {
    use crate::jvm::command::JvmCommand;
    use crate::jvm::direct::DirectJvm;
    use crate::jvm::process::ProcessJvm;
    use crate::jvm::JvmTask;

    #[test]
    fn spawn_jvm() {
        let jvm = ProcessJvm::new();
        let jvm = DirectJvm::spawn(jvm).expect("Failed to spawn JVM");

        let mut task = JvmCommand::new(&jvm, "net.dblsaiko.origami.TestService")
            .arg("arg1")
            .spawn()
            .expect("Failed to spawn task");

        task.wait().expect("Failed to wait for task to exit");
    }
}
