export default {
    error: {
        name_already_exists: 'Name already exists: {name}',
        valid_tls_certificates_not_found: 'Unable to find valid TLS certificates',
        cert_already_exists: 'Certificate already exists: {id}',
        failed_to_read_cert: 'Failed to read certificate',
        failed_to_read_private_key: 'Failed to read private key',
    },
    ports: {
        ports: 'Ports',
        no_ports: 'No port configurations found.',
        new_port: 'New Port',
        delete_port: 'Delete Port',
        delete_port_confirm: 'Are you sure to delete {name}?',
        successfully_updated: 'Successfully updated',
        snackbar_close: 'Close',
        state: {
            listening: 'Listening',
            port_already_in_use: 'Port already in use',
            permission_denied: 'Permission denied',
            address_not_available: 'Address not available',
            no_valid_certificate: 'No valid certificate',
            configuration_failed: 'Configuration failed',
            error: 'Error',
            unknown: 'Unknown',
        },
        tab: {
            status: 'Status',
            settings: 'Settings',
        },
        status: {
            name: 'Name',
            interface: 'Interface',
            state: 'State',
            uptime: 'Uptime',
        },
        config: {
            name: 'Name',
            protocol: 'Protocol',
            interface: 'Interface',
            port: 'Port',
            host: 'Host',
            servers: 'Servers',
            cancel: 'Cancel',
            update: 'Update',
            create: 'Create',
            rule: {
                name_required: 'A name is required.',
                interface_required: 'Enter a valid IPv4 or IPv6 address.',
                hostname_required: 'Provide a valid hostname or IPv4/IPv6 address.',
                port_required: 'Enter a valid port number (range: 1-65535).',
            },
            tls_term: {
                tls_term: "TLS Termination",
                server_names: {
                    server_names: "Server Names",
                    hint: "You can use commas to list multiple names, e.g, example.com, *.test.examle.com.",
                    rule: "Enter valid server names.",
                }
            }
        }
    },
    certs: {
        certs: 'Certificates',
        info: {
            san: 'SAN',
            fingerprint: 'Fingerprint',
            issuer: 'Issuer',
            root_cert: 'Root Certificate',
            not_before: 'Not Before',
            not_after: 'Not After',
        },
        add_cert: "Add Certificate",
        self_sign: {
            self_sign: "Self-sign",
            title: "New Self-signed Certificate",
            subject_alternative_names: "Subject Alternative Names",
            hint: "You can use commas to list multiple names, e.g, example.com, *.test.examle.com.",
            rule: "Enter valid subject alternative names.",
            create: "Create",
            cancel: "Cancel"
        },
        upload: {
            upload: "Upload",
            title: "Upload Certificate",
            chain: "Certificate Chain",
            key: "Private Key",
            hint: "Only PEM format is supported.",
            rule_chain: "Select valid certificate chain.",
            rule_key: "Select valid private key.",
            cancel: "Cancel"
        },
        delete_cert: {
            delete_cert: "Delete Certificate",
            confirm: "Are you sure to delete {name}?",
            delete: "Delete",
            cancel: "Cancel"
        }
    }
}