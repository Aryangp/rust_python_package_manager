use std::{collections::HashMap, fmt::format};
use std::error::Error;
use std::fs;
use std::path::Path;
use serde::{Deserialize,Serialize};

use serde_json;

#[derive(Debug,Deserialize,Serialize)]
struct Package {
    name:String,
    version:String,
    dependencies: Vec<String>
}

#[derive(Debug)]
struct DependencyManager {
    packages: HashMap<String,Package>,
}

impl DependencyManager {
    fn new()-> Self {
        DependencyManager {
            packages: HashMap::new(),
        }
    }

    fn add_package(&mut self, package:Package) {
        self.packages.insert(package.name.clone(),package);
    }

    fn get_package(&self , name:&str)-> Option<&Package>{
        self.packages.get(name)
    }

    fn resolve_dependencies(&self, package_name:&str)->Result<Vec<String>,Box<dyn Error>>{
        let mut resolved = Vec::new();
        let mut to_visit = vec![package_name.to_string()];

        while let Some(name) = to_visit.pop() {
            if !resolved.contains(&name) {
                let package = self.get_package(&name).ok_or_else(|| format!("package not found {}",name))?;
                resolved.push(name);
                to_visit.extend(package.dependencies.clone());
            }
        }
        Ok(resolved)
    }

    fn load_from_file<P: AsRef<Path>>(&mut self,path:P)-> Result<(),Box<dyn Error>>{
        let contents = fs::read_to_string(path)?;
        let packages:Vec<Package> = serde_json::from_str(&contents)?;
        
        for package in packages {
            self.add_package(package);
        }

        Ok(())
    }

    fn save_to_file<P:AsRef<Path>>(&mut self , path:P)->Result<(),Box<dyn Error>>{
        let packages:Vec<&Package> = self.packages.values().collect();
        let contents= serde_json::to_string_pretty(&packages)?;
        fs::write(path,contents)?;
        Ok(())
    }
}


fn main() {
    let mut manager = DependencyManager::new();
    manager.add_package(Package { 
        name: "flask".to_string(), 
        version: "2.1.0".to_string(), 
        dependencies: vec!["werkzeug".to_string(), "jinja2".to_string()] 
    });
    manager.add_package(Package {
        name: "werkzeug".to_string(),
        version: "2.0.1".to_string(),
        dependencies: vec![],
    });

    manager.add_package(Package {
        name: "jinja2".to_string(),
        version: "3.0.1".to_string(),
        dependencies: vec!["markupsafe".to_string()],
    });

    manager.add_package(Package {
        name: "markupsafe".to_string(),
        version: "2.0.1".to_string(),
        dependencies: vec![],
    });

    let resolved =  manager.resolve_dependencies("flask")?;
    println!("Resolved dependencies: {:?}", resolved);

    manager.save_to_file("package.json");

    let mut  new_manager = DependencyManager::new();
    new_manager.load_from_file("package.json");

}

