export function isValidHostname(hostname) {
    const validHostnameRegex = /^(?!\-)[A-Za-z0-9\-]{1,63}(?<!\.)\.((?!\-)[A-Za-z0-9\-]{1,63}(?<!\-)\.?)+$/;
    return hostname === 'localhost' || (validHostnameRegex.test(hostname) && hostname.length <= 253);
}

export function isValidTlsServerName(name) {
    const nameWithoutAsterisk = name.replace(/^\*\./, '')
    if (isValidHostname(nameWithoutAsterisk)) return true
    try {
        new Address4(name)
        return true
    } catch (_) { }
    try {
        new Address6(name)
        return true
    } catch (_) { }
    return false
}

export function parseTlsServerNames(names) {
    const list = names.split(',').map(n => n.trim())
    if (list.some(n => !isValidTlsServerName(n))) return []
    return list
}