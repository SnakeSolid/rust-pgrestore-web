"use strict";

requirejs.config({
	baseUrl: "/static/js",
	paths: {
		knockout: ["https://cdnjs.cloudflare.com/ajax/libs/knockout/3.4.2/knockout-min", "lib/knockout-min"],
		moment: ["https://cdnjs.cloudflare.com/ajax/libs/moment.js/2.22.2/moment.min", "lib/moment.min"],
		reqwest: ["https://cdnjs.cloudflare.com/ajax/libs/reqwest/2.0.5/reqwest.min", "lib/reqwest.min"],
		semantic: ["https://cdnjs.cloudflare.com/ajax/libs/semantic-ui/2.4.1/semantic.min", "lib/semantic.min"],
		text: ["https://cdnjs.cloudflare.com/ajax/libs/require-text/2.0.12/text.min", "lib/text.min"],
	},
	shim: {
		reqwest: {
			exports: "reqwest",
		},
	},
	waitSeconds: 15,
});

// Start the main application logic.
requirejs(
	["knockout", "Application"],
	function(ko, Application) {
		const application = new Application();

		ko.applyBindings(application);
	},
	function(err) {
		console.error(err.requireType);

		if (err.requireType === "timeout") {
			console.error("modules: " + err.requireModules);
		}

		throw err;
	}
);
