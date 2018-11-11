"use strict";

define([ "knockout" ], function(ko) {
	ko.components.register("ko-restore", {
		viewModel: { require: "components/Restore" },
		template: { require: "text!components/Restore.html" }
	});

	ko.components.register("ko-status", {
		viewModel: { require: "components/Status" },
		template: { require: "text!components/Status.html" }
	});
});
