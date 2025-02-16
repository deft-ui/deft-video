export function formatHumanShortTime(value) {
    let seconds = (value % 60).toFixed(0).padStart(2, '0');
    value = Math.floor(value / 60);
    let minutes = (value % 60).toFixed(0).padStart(2, '0');
    value = Math.floor(value / 60);
    let hours = value ? (value % 60).toFixed(0).padStart(2, '0') : "";
    return [hours, minutes, seconds].filter(Boolean).join(":");
}