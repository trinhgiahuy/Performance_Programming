use rpkg::debversion;
use crate::Packages;
use crate::packages::Dependency;

impl Packages {
    /// Gets the dependencies of package_name, and prints out whether they are satisfied (and by which library/version) or not.
    pub fn deps_available(&self, package_name: &str) {
        if !self.package_exists(package_name) {
            println!("no such package {}", package_name);
            return;
        }

        let deps : &Vec<Dependency> = &*self.dependencies.get(self.get_package_num(package_name)).unwrap();

        println!("Package {}:", package_name);
        for dep in deps {
            println!("- dependency {:?}", self.dep2str(dep));
            if let Some(pkg) = self.dep_is_satisfied(dep) {
                let installed_version = self.get_installed_debver(pkg).unwrap();
                println!("+ {} satisfied by installed version {}", pkg, installed_version);
            } else {
                println!("-> not satisfied");
            }
        }
    }

    /// Returns Some(package) which satisfies dependency dd, or None if not satisfied.
    pub fn dep_is_satisfied(&self, dd:&Dependency) -> Option<&str> {
        // presumably you should loop on dd

        for dep in dd {
            let package_name = self.get_package_name(dep.package_num);
            let installed_debver = self.get_installed_debver(package_name);

            if installed_debver.is_some() {
                let installed_version = installed_debver.unwrap();

                if dep.rel_version.is_none() {
                    return Some(package_name);
                } else {
                    let rel_version = dep.rel_version.as_ref().unwrap();
                    let op = &rel_version.0;
                    let ver = &rel_version.1;
                    let ver_debver = ver.parse::<debversion::DebianVersionNum>().unwrap();

                    if debversion::cmp_debversion_with_op(op, installed_version, &ver_debver) {
                        return Some(package_name);
                    } else {
                        continue;
                    }
                }
            }
        }

        return None;
    }

    /// Returns a Vec of packages which would satisfy dependency dd but for the version.
    /// Used by the how-to-install command, which calls compute_how_to_install().
    pub fn dep_satisfied_by_wrong_version(&self, dd:&Dependency) -> Vec<&str> {
        assert! (self.dep_is_satisfied(dd).is_none());
        let mut result = vec![];
        // another loop on dd

        for dep in dd {
            let package_name = self.get_package_name(dep.package_num);
            let installed_debver = self.get_installed_debver(package_name);

            if installed_debver.is_some() {
                let installed_version = installed_debver.unwrap();

                if dep.rel_version.is_none() {
                    continue;
                } else {
                    let rel_version = dep.rel_version.as_ref().unwrap();
                    let op = &rel_version.0;
                    let ver = &rel_version.1;
                    let ver_debver = ver.parse::<debversion::DebianVersionNum>().unwrap();
                    if !debversion::cmp_debversion_with_op(op, installed_version, &ver_debver) {
                        result.push(package_name);
                    }
                }
            }
        }

        return result;
    }
}

