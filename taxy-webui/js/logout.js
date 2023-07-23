export function logout() {
    if (location.pathname !== "/login") {
        location.pathname = "/login";
    }
}