//! Port allocation and conflict resolution
//!
//! Port ranges by service type:
//! - Web Frontends (SvelteKit, Next.js, React): 3000-3099
//! - APIs (REST, GraphQL): 8000-8099
//! - Database ports: 5432 (Postgres), 3306 (MySQL), 27017 (Mongo)
//! - Admin/Debug: 9000-9099

use std::collections::HashMap;
use std::ops::RangeInclusive;

use crate::manifest::ServiceType;
use crate::process::is_port_in_use;
use crate::registry::Registry;

/// Port range definitions for different service types
#[derive(Debug, Clone)]
pub struct PortRange {
    pub name: &'static str,
    pub range: RangeInclusive<u16>,
    pub description: &'static str,
}

/// Predefined port ranges
pub const PORT_RANGES: &[PortRange] = &[
    PortRange {
        name: "frontend",
        range: 3000..=3099,
        description: "Web Frontends (SvelteKit, Next.js, React, Vue)",
    },
    PortRange {
        name: "api",
        range: 8000..=8099,
        description: "APIs (REST, GraphQL, gRPC)",
    },
    PortRange {
        name: "admin",
        range: 9000..=9099,
        description: "Admin/Debug/Internal services",
    },
    PortRange {
        name: "flutter",
        range: 3100..=3199,
        description: "Flutter web apps",
    },
    PortRange {
        name: "rust",
        range: 8100..=8199,
        description: "Rust services",
    },
];

/// Get the appropriate port range for a service type
#[allow(dead_code)]
pub fn get_range_for_type(service_type: &ServiceType) -> &'static PortRange {
    match service_type {
        ServiceType::Node => &PORT_RANGES[0],          // frontend
        ServiceType::Python => &PORT_RANGES[1],        // api
        ServiceType::Go => &PORT_RANGES[1],            // api
        ServiceType::Dart => &PORT_RANGES[1],          // api (Dart backend services)
        ServiceType::RustBinary => &PORT_RANGES[4],    // rust
        ServiceType::DockerCompose => &PORT_RANGES[2], // admin
        ServiceType::Shell => &PORT_RANGES[2],         // admin
    }
}

/// Collect all allocated ports from the registry
pub fn collect_allocated_ports(registry: &Registry) -> HashMap<u16, Vec<(String, String)>> {
    let mut port_map: HashMap<u16, Vec<(String, String)>> = HashMap::new();

    for (name, entry) in registry.list() {
        let manifest_path = entry.path.join("devhub.toml");
        if let Ok(manifest) = crate::manifest::Manifest::load(&manifest_path) {
            for service in &manifest.services {
                port_map
                    .entry(service.port)
                    .or_default()
                    .push((name.clone(), service.name.clone()));
            }
        }
    }

    port_map
}

/// Find conflicts (ports used by multiple services)
pub fn find_conflicts(
    port_map: &HashMap<u16, Vec<(String, String)>>,
) -> Vec<(u16, Vec<(String, String)>)> {
    port_map
        .iter()
        .filter(|(_, services)| services.len() > 1)
        .map(|(port, services)| (*port, services.clone()))
        .collect()
}

/// Suggest an available port in a given range
pub fn suggest_port_in_range(
    range: &PortRange,
    allocated_ports: &HashMap<u16, Vec<(String, String)>>,
) -> Option<u16> {
    range
        .range
        .clone()
        .find(|&port| !allocated_ports.contains_key(&port) && !is_port_in_use(port))
}

/// Suggest alternative ports for conflicts
pub fn suggest_alternatives(
    conflicts: &[(u16, Vec<(String, String)>)],
    allocated_ports: &HashMap<u16, Vec<(String, String)>>,
) -> Vec<PortConflictResolution> {
    let mut resolutions = Vec::new();

    for (port, services) in conflicts {
        // Keep the first service, suggest moves for others
        let keeper = &services[0];
        let movers: Vec<_> = services.iter().skip(1).cloned().collect();

        for (project, service) in &movers {
            // Find the service type to suggest an appropriate range
            let suggested_range = find_range_for_port(*port);

            if let Some(new_port) = suggest_port_in_range(suggested_range, allocated_ports) {
                resolutions.push(PortConflictResolution {
                    original_port: *port,
                    project: project.clone(),
                    service: service.clone(),
                    suggested_port: new_port,
                    keeper_project: keeper.0.clone(),
                    keeper_service: keeper.1.clone(),
                });
            }
        }
    }

    resolutions
}

/// Find which range a port belongs to
fn find_range_for_port(port: u16) -> &'static PortRange {
    for range in PORT_RANGES {
        if range.range.contains(&port) {
            return range;
        }
    }
    // Default to frontend range if not found
    &PORT_RANGES[0]
}

/// A suggested resolution for a port conflict
#[derive(Debug, Clone)]
pub struct PortConflictResolution {
    pub original_port: u16,
    pub project: String,
    pub service: String,
    pub suggested_port: u16,
    pub keeper_project: String,
    pub keeper_service: String,
}

impl PortConflictResolution {
    /// Generate a command to fix this conflict
    pub fn fix_command(&self) -> String {
        format!(
            "# In {}/devhub.toml, change port {} to {} for service '{}'",
            self.project, self.original_port, self.suggested_port, self.service
        )
    }
}

/// Allocate a new port for a project/service
#[allow(dead_code)]
pub fn allocate_port(service_type: &ServiceType, registry: &Registry) -> Option<u16> {
    let range = get_range_for_type(service_type);
    let allocated = collect_allocated_ports(registry);
    suggest_port_in_range(range, &allocated)
}

/// Port allocation result with details
#[allow(dead_code)]
#[derive(Debug)]
pub struct PortAllocation {
    pub port: u16,
    pub range_name: &'static str,
    pub range_description: &'static str,
}

/// Allocate a port with full details
#[allow(dead_code)]
pub fn allocate_port_detailed(
    service_type: &ServiceType,
    registry: &Registry,
) -> Option<PortAllocation> {
    let range = get_range_for_type(service_type);
    let allocated = collect_allocated_ports(registry);

    suggest_port_in_range(range, &allocated).map(|port| PortAllocation {
        port,
        range_name: range.name,
        range_description: range.description,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_ranges_are_valid() {
        for range in PORT_RANGES {
            assert!(range.range.start() <= range.range.end());
        }
    }

    #[test]
    fn test_ranges_dont_overlap() {
        for (i, range1) in PORT_RANGES.iter().enumerate() {
            for range2 in PORT_RANGES.iter().skip(i + 1) {
                let overlap = range1.range.start() <= range2.range.end()
                    && range2.range.start() <= range1.range.end();
                assert!(
                    !overlap,
                    "Ranges {} and {} overlap",
                    range1.name, range2.name
                );
            }
        }
    }
}
