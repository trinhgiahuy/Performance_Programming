use crate::packages::Dependency;
use crate::Packages;

use super::RelVersionedPackageNum;
use crate::debversion;
use std::collections::VecDeque;

impl Packages {
    /// Computes a solution for the transitive dependencies of package_name; when there is a choice A | B | C,
    /// chooses the first option A. Returns a Vec<i32> of package numbers.
    ///
    /// Note: does not consider which packages are installed.
    pub fn transitive_dep_solution(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }

        let deps: &Vec<Dependency> = &*self
            .dependencies
            .get(self.get_package_num(package_name))
            .unwrap();
        let mut dependency_set = vec![];
        
        // ? Iterate the first alternative given assuming it is installable
        for dep in deps {
            if let Some(item) = dep.first() {
                dependency_set.push(item.package_num);
            }
        }

        let mut new_dependencies_added = true;

        while new_dependencies_added {
            new_dependencies_added = false;

            let current_dependencies = dependency_set.clone();

            for pkg_num in current_dependencies {
                if let Some(deps) = self.dependencies.get(&pkg_num) {
                    for dep in deps {
                        if let Some(item) = dep.first() {
                            if !dependency_set.contains(&item.package_num) {
                                dependency_set.push(item.package_num);
                                new_dependencies_added = true;
                            }
                        }
                    }
                }
            }
        }

        return dependency_set;
    }

    /// Computes a set of packages that need to be installed to satisfy package_name's deps given the current installed packages.
    /// When a dependency A | B | C is unsatisfied, there are two possible cases:
    ///   (1) there are no versions of A, B, or C installed; pick the alternative with the highest version number (yes, compare apples and oranges).
    ///   (2) at least one of A, B, or C is installed (say A, B), but with the wrong version; of the installed packages (A, B), pick the one with the highest version number.
    pub fn compute_how_to_install(&self, package_name: &str) -> Vec<i32> {
        if !self.package_exists(package_name) {
            return vec![];
        }

        let mut dependencies_to_add: Vec<i32> = vec![];

        let deps = self
            .dependencies
            .get(&self.get_package_num(package_name))
            .unwrap();
        let mut remaining_deps = VecDeque::new();
        remaining_deps.extend(deps.iter());

        while let Some(current_dependency) = remaining_deps.pop_front() {
            if self.dep_is_satisfied(current_dependency).is_some() {
                continue;
            }

            let wrong_version_packages = self.dep_satisfied_by_wrong_version(current_dependency);

            if wrong_version_packages.is_empty() {
                if current_dependency.len() == 1 {
                    let dep_num = current_dependency[0].package_num;
                    if !dependencies_to_add.contains(&dep_num) {
                        dependencies_to_add.push(dep_num);
                        self.dependencies
                            .get(&dep_num)
                            .unwrap()
                            .iter()
                            .for_each(|md| remaining_deps.push_back(md));
                    }
                } else {
                    let best_dep = current_dependency
                        .iter()
                        .map(|el| {
                            let ver = el
                                .rel_version
                                .as_ref()
                                .unwrap()
                                .1
                                .parse::<debversion::DebianVersionNum>()
                                .unwrap();
                            (el, ver)
                        })
                        .max_by(|(_, v1), (_, v2)| v1.cmp(v2))
                        .unwrap();

                    if !dependencies_to_add.contains(&best_dep.0.package_num) {
                        dependencies_to_add.push(best_dep.0.package_num);
                        self.dependencies
                            .get(&best_dep.0.package_num)
                            .unwrap()
                            .iter()
                            .for_each(|md| remaining_deps.push_back(md));
                    }
                }
            } else {
                let dep_num = wrong_version_packages
                    .iter()
                    .map(|name| self.get_package_num(name))
                    .max_by(|&a, &b| a.cmp(&b))
                    .unwrap();

                if !dependencies_to_add.contains(&dep_num) {
                    dependencies_to_add.push(*dep_num);
                    self.dependencies
                        .get(&dep_num)
                        .unwrap()
                        .iter()
                        .for_each(|md| remaining_deps.push_back(md));
                }
            }
        }

        dependencies_to_add
    }
}
