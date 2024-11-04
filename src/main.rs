use std::process::Command;
use std::{collections::HashMap, fmt::format};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize,Serialize};


//   todos for this project are 
// 1 make sure that the requirements file is created and updated on every new package installation
// 2 it get made from pip freeze command 





#[derive(Debug)]
struct PythonEnvManager{
    base_path:PathBuf,
    dependencies:HashMap<String,Package>,
}

#[derive(Debug,Deserialize,Serialize)]
struct Package {
    name:String,
    version:String,
    dependencies: Vec<String>
}

#[derive(Debug)]
enum EnvError {
    VenvCreationError(String),
    PipInstallError(String),
    PathError(String),
   
}

impl std::fmt::Display for EnvError {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        match self {
            EnvError::VenvCreationError(e) => write!(f,"Error creating virtual environment: {}",e),
            EnvError::PipInstallError(e) => write!(f,"Error installing package: {}",e),
            EnvError::PathError(e) => write!(f,"Error with path: {}",e),
        }
    }
}

impl Error for EnvError {}

impl PythonEnvManager {
    fn new<P:AsRef<Path>>(base_path:P)-> Result<Self,Box<dyn Error>>  {
        let base_path = base_path.as_ref().to_path_buf();
        fs::create_dir_all(&base_path)?;

        Ok(PythonEnvManager{
            base_path,
            dependencies:HashMap::new(),
        })
    }

    fn create_virtual_env(&self, project_name:&str)-> Result<PathBuf,EnvError>{
        let venv_path = self.base_path.join(format!("{}/.venv",project_name));
        let output = Command::new("python")
            .args(&["-m","venv",venv_path.to_str().unwrap()])
            .output()
            .map_err(|err| EnvError::VenvCreationError(err.to_string()))?;

        if !output.status.success() {
            return Err(EnvError::VenvCreationError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(venv_path)
    }

    fn get_pip_path(&self, venv_path:&Path)-> PathBuf{
        #[cfg(windows)]
        let pip_path = venv_path.join("Scripts").join("pip.exe");

        #[cfg(not (windows))]
        let pip_path = venv_path.join("bin").join("pip");

        pip_path
    }

    fn install_package(&self, venv_path:&Path , package:&str , version:Option<&str>)->Result<(),EnvError>{
       let pip_path = self.get_pip_path(venv_path);

       let package_spec = match version {
        Some(v)=> format!("{}=={}",package,v),
        None => package.to_string(),
       };

       let output = Command::new(pip_path)
            .args(&["install","--quiet",package_spec.as_str()])
            .output()
            .map_err(|err| EnvError::PipInstallError(err.to_string()))?;

       
       if !output.status.success(){
        return Err(EnvError::PipInstallError(
            String::from_utf8_lossy(&output.stderr).to_string()
        ));
       }

        Ok(()) 
    }


    fn upgrade_pip(&self , venv_path:&Path) -> Result<(),EnvError> {
        let pip_path = self.get_pip_path(venv_path);

        let output = Command::new(&pip_path)
            .args(&["install","--upgrade","pip"])
            .output()
            .map_err(|err| EnvError::PipInstallError(err.to_string()))?;

        if !output.status.success() {
            return Err(EnvError::PipInstallError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(())
    }

    fn create_requirements_file(&self,venv_path:&Path)-> Result<(),EnvError>{
        let req_path = venv_path.parent()
            .ok_or_else(|| EnvError::PathError(String::from("Error getting the parent path")))?
            .to_path_buf()
            .join("requirements.txt");

 

        let output = Command::new(self.get_pip_path(venv_path))
            .args(&["freeze"])
            .stdout(fs::File::create(req_path).unwrap())
            .output()
            .map_err(|err| EnvError::PipInstallError(err.to_string()))?;

        if !output.status.success() {
            return Err(EnvError::PipInstallError(
                String::from_utf8_lossy(&output.stderr).to_string()
            ));
        }

        Ok(())
 
    }


    fn setup_project(&self, project_name:&str , packages:&[(&str,Option<&str>)])-> Result<PathBuf,EnvError>{
        let venv_path = self.create_virtual_env(project_name)?;
        println!("Created virtual environment at {:?}",venv_path);

        for (package,version) in packages {
            println!("Installing {}=={}",package,version.unwrap_or("latest"));
            self.install_package(&venv_path,package,*version)?;
        }

        self.create_requirements_file(&venv_path)?;
        print!("created requirements file");

        Ok(venv_path)
       
    }

   
}


fn main()-> Result<(),Box<dyn Error>> {
    let mut manager = PythonEnvManager::new("./python_project")?;

    let packages = vec![
        ("requests",None),
        ("flask",None),
       
    ];

    match manager.setup_project("my_first_project", &packages) {
        Ok(venv_path) => println!("Project setup successfully at {:?}",venv_path),
        Err(e) => eprintln!("Error setting up project: {}",e),
    }

    Ok(())

}

