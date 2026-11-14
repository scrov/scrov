// Small behaviours shared by every page. Menus and selection states are
// written as hyperscript on the elements themselves; this file holds the two
// things that need real JavaScript.

/** Apply a distro theme and remember it. Called from hyperscript. */
function scrovTheme(theme) {
	document.documentElement.dataset.theme = theme;
	try {
		localStorage.setItem("theme", theme);
	} catch {
		// private mode: the theme still applies for this page view
	}
	const label = document.querySelector("[data-current-theme]");
	if (label) label.textContent = theme;
	for (const option of document.querySelectorAll("[data-theme-option]")) {
		option.setAttribute(
			"aria-selected",
			String(option.dataset.themeOption === theme),
		);
	}
}

function formatUptime(ms) {
	const total = Math.max(0, Math.floor(ms / 1000));
	const days = Math.floor(total / 86400);
	const hours = Math.floor((total % 86400) / 3600);
	const minutes = Math.floor((total % 3600) / 60);
	const seconds = total % 60;
	const plural = (n, unit) => `${n} ${unit}${n === 1 ? "" : "s"}`;

	const parts = [];
	if (days > 0) parts.push(plural(days, "day"));
	if (hours > 0) parts.push(plural(hours, "hour"));
	if (minutes > 0) parts.push(plural(minutes, "minute"));
	if (parts.length === 0) parts.push(plural(seconds, "second"));
	return parts.join(", ");
}

function tickUptime() {
	for (const el of document.querySelectorAll("[data-uptime]")) {
		const start = new Date(el.dataset.uptime).getTime();
		if (!Number.isNaN(start)) el.textContent = formatUptime(Date.now() - start);
	}
}

document.addEventListener("DOMContentLoaded", () => {
	scrovTheme(document.documentElement.dataset.theme);
	tickUptime();
	setInterval(tickUptime, 1000);
});
