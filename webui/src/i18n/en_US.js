export default {
    error: {
        valid_tls_certificates_not_found: 'Unable to find valid TLS certificates',
        cert_already_exists: 'Certificate already exists: {id}',
        failed_to_read_cert: 'Failed to read certificate',
        failed_to_read_private_key: 'Failed to read private key',
    },
    login: {
        login: 'Login',
        username: 'Username',
        password: 'Password',
        login_failed: 'Login failed',
        username_required: 'Username is required',
        password_required: 'Password is required',
    },
    ports: {
        ports: 'Ports',
        no_ports: 'No port configurations found.',
        new_port: 'New Port',
        delete_port: 'Delete Port',
        delete_port_confirm: 'Are you sure to delete {id}?',
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
            interface: 'Interface',
            state: 'State',
            uptime: 'Uptime',
        },
        config: {
            listener: 'Listener',
            protocol: 'Protocol',
            interface: 'Interface',
            port: 'Port',
            host: 'Host',
            url: 'URL',
            servers: 'Servers',
            cancel: 'Cancel',
            update: 'Update',
            create: 'Create',
            rule: {
                interface_required: 'Enter a valid IPv4 or IPv6 address.',
                hostname_required: 'Provide a valid hostname or IPv4/IPv6 address.',
                port_required: 'Enter a valid port number (range: 1-65535).',
                url_required: 'Enter a valid HTTP/HTTPS URL.',
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
    _keyring: {
        keyring: 'Keyring',
        no_certs: 'No keyring item found.',
        info: {
            san: 'SAN',
            fingerprint: 'Fingerprint',
            issuer: 'Issuer',
            root_cert: 'Root Certificate',
            not_before: 'Not Before',
            not_after: 'Not After',
        },
        add_item: "Add",
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
        acme: {
            acme: "ACME",
            title: "New ACME Request",
            provider: "Provider",
            challenge: "Challenge",
            email: "Email",
            domain: "Domain Name",
            create: "Create",
            cancel: "Cancel",
            rule: {
                hostname_required: 'Provide a valid hostname.',
            }
        },
        delete_cert: {
            delete_cert: "Delete Certificate",
            confirm: "Are you sure to delete {id}?",
            delete: "Delete",
            cancel: "Cancel"
        },
        delete_acme: {
            delete_acme: "Delete ACME",
            confirm: "Are you sure to delete {id}?",
            cancel: "Cancel",
            delete: "Delete"
        }
    },
    acme: {
        acme: "ACME",
        add_item: "Add",
        no_items: "No ACME requests found.",
        provider: "Provider",
        challenge: "Challenge",
        domain: "Domain Name",
        create: "Create",
        cancel: "Cancel",
        add_acme: {
            title: "New ACME Request",
            cancel: "Cancel",
            create: "Create",
        },
        delete_acme: {
            delete_acme: "Delete ACME",
            confirm: "Are you sure to delete {id}?",
            cancel: "Cancel",
            delete: "Delete"
        },
        info: {
            provider: "Provider",
            identifiers: "Identifiers",
            challenge_type: "Challenge Type",
        }
    }
}