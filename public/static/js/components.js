"use strict";

define(["knockout"], function(ko) {
	ko.components.register("ko-search", {
		viewModel: { require: "components/Search" },
		template: { require: "text!components/Search.html" },
	});

	ko.components.register("ko-restore", {
		viewModel: { require: "components/Restore" },
		template: { require: "text!components/Restore.html" },
	});

	ko.components.register("ko-status", {
		viewModel: { require: "components/Status" },
		template: { require: "text!components/Status.html" },
	});

	ko.components.register("ko-jobs", {
		viewModel: { require: "components/Jobs" },
		template: { require: "text!components/Jobs.html" },
	});

	ko.components.register("ko-settings", {
		viewModel: { require: "components/Settings" },
		template: { require: "text!components/Settings.html" },
	});

	// Dialogs

	ko.components.register("ko-parse-dialog", {
		viewModel: { require: "components/ParseDialog" },
		template: { require: "text!components/ParseDialog.html" },
	});

	ko.components.register("ko-export-dialog", {
		viewModel: { require: "components/ExportDialog" },
		template: { require: "text!components/ExportDialog.html" },
	});

	ko.components.register("ko-import-dialog", {
		viewModel: { require: "components/ImportDialog" },
		template: { require: "text!components/ImportDialog.html" },
	});
});
