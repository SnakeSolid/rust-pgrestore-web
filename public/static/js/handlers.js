"use strict";

define(["knockout"], function(ko) {
	ko.bindingHandlers.enterkey = {
		init: function(element, valueAccessor, allBindings, viewModel) {
			const callback = valueAccessor();

			element.addEventListener("keypress", function(event) {
				if (event.key === "Enter") {
					callback.call(viewModel);

					return false;
				}

				return true;
			});
		},
	};
});
