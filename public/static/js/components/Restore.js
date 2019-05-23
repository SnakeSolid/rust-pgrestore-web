"use strict";

define(["knockout", "reqwest", "Storage", "Pattern"], function(ko, reqwest, Storage, Pattern) {
	const BACKUP_PATH = "Path";
	const BACKUP_URL = "Url";
	const DATABASE_EXISTS = "Exists";
	const DATABASE_DROPANDCREATE = "DropAndCreate";
	const RESTORE_FULL = "Full";
	const RESTORE_PARTIAL = "Partial";

	const EXTRACT_TABLES_RES = [
		/insert\s+into\s+(\w+\.\w+)\b/gi,
		/update\s+(\w+\.\w+)\b/gi,
		/from\s+(\w+\.\w+)\b/gi,
		/join\s+(\w+\.\w+)\b/gi,
	];
	const SEPARATORS_RE = /[\s,]+/;
	const WORDS_RE = /\w+/;

	const nonEmptyString = function(value) {
		return value.length > 0;
	};

	const Restore = function(params) {
		this.backup = params.backup;
		this.restoreCallback = params.restoreCallback;
		this.isIndexesVisible = params.isIndexesVisible;

		this.backup.subscribe(this.inferDatabaseName.bind(this));

		this.availableDestinations = params.destinations;
		this.selectedDestination = ko.observable();
		this.backupType = ko.observable(BACKUP_PATH);
		this.databaseName = ko.observable("");
		this.database = ko.observable(DATABASE_DROPANDCREATE);
		this.restore = ko.observable(RESTORE_FULL);
		this.objects = ko.observable("");
		this.isRestoreSchema = ko.observable(false);
		this.isRestoreIndexes = ko.observable(params.isIndexesVisible());
		this.ignoreErrors = ko.observable(false);
		this.parseSchemaVisible = ko.observable(false);
		this.parseTablesVisible = ko.observable(false);
		this.isLoading = ko.observable(false);
		this.isError = ko.observable(false);
		this.errorMessage = ko.observable();

		this.isBackupPath = ko.pureComputed(function() {
			return this.backupType() === BACKUP_PATH;
		}, this);

		this.isBackupUrl = ko.pureComputed(function() {
			return this.backupType() === BACKUP_URL;
		}, this);

		this.isDestinationInvalid = ko.pureComputed(function() {
			return this.selectedDestination() === undefined;
		}, this);

		this.isBackupPathInvalid = ko.pureComputed(function() {
			return this.backup().length === 0;
		}, this);

		this.isDatabaseNameInvalid = ko.pureComputed(function() {
			return this.databaseName().length === 0;
		}, this);

		this.isRestoreInvalid = ko.pureComputed(function() {
			return this.isRestorePartial() && !WORDS_RE.test(this.objects());
		}, this);

		this.isRestoreFull = ko.pureComputed(function() {
			return this.restore() === RESTORE_FULL;
		}, this);

		this.isRestorePartial = ko.pureComputed(function() {
			return this.restore() === RESTORE_PARTIAL;
		}, this);

		this.isFormInvalid = ko.pureComputed(function() {
			return (
				this.isDestinationInvalid() ||
				this.isBackupPathInvalid() ||
				this.isDatabaseNameInvalid() ||
				this.isRestoreInvalid()
			);
		}, this);

		this.schemaCallback = function(text) {
			this.restore(RESTORE_PARTIAL);
			this.objects(
				this.parseSchema(text.toLowerCase())
					.sort()
					.join(", ")
			);
		}.bind(this);

		this.tablesCallback = function(text) {
			this.restore(RESTORE_PARTIAL);
			this.objects(
				this.parseTables(text.toLowerCase())
					.sort()
					.join(", ")
			);
		}.bind(this);

		this.backup.subscribe(
			function(value) {
				if (value.startsWith("http://") || value.startsWith("https://")) {
					this.backupType(BACKUP_URL);
				} else {
					this.backupType(BACKUP_PATH);
				}
			}.bind(this)
		);

		this.updateSelectedDestination();
		this.availableDestinations.subscribe(this.updateSelectedDestination);
	};

	Restore.prototype.updateSelectedDestination = function() {
		this.selectedDestination(Storage.getPreferredDestination());
	};

	Restore.prototype.inferDatabaseName = function(backupPath) {
		const databaseName = Pattern.inferDatabaseName(backupPath);

		if (databaseName !== undefined) {
			this.databaseName(databaseName);
		}
	};

	Restore.prototype.backupToCall = function() {
		const result = {};

		if (this.isBackupPath()) {
			result.type = BACKUP_PATH;
			result.path = this.backup();
		} else if (this.isBackupUrl()) {
			result.type = BACKUP_URL;
			result.url = this.backup();
		}

		return result;
	};

	Restore.prototype.restoreToCall = function() {
		const result = {};

		if (this.isRestoreFull()) {
			result.type = RESTORE_FULL;
		} else if (this.isRestorePartial()) {
			result.type = RESTORE_PARTIAL;
			result.objects = this.objects()
				.split(SEPARATORS_RE)
				.filter(nonEmptyString);
			result.restore_schema = this.isRestoreSchema();
			result.restore_indexes = this.isRestoreIndexes();
		}

		return result;
	};

	Restore.prototype.restoreDatabase = function() {
		reqwest({
			url: "/api/v3/restore",
			type: "json",
			method: "POST",
			contentType: "application/json",
			data: JSON.stringify({
				destination: this.selectedDestination(),
				backup: this.backupToCall(),
				database_name: this.databaseName(),
				database: this.database(),
				restore: this.restoreToCall(),
				ignore_errors: this.ignoreErrors(),
			}),
		})
			.then(
				function(resp) {
					if (resp.success) {
						this.restoreCallback(resp.result);
						this.isError(false);
					} else {
						this.isError(true);
						this.errorMessage(resp.message);
					}

					this.isLoading(false);
				}.bind(this)
			)
			.fail(
				function(err, msg) {
					this.isLoading(false);
					this.isError(true);
					this.errorMessage(msg || err.responseText);
				}.bind(this)
			);

		this.isLoading(true);
	};

	Restore.prototype.convertSlashes = function() {
		const backup = this.backup();
		const nForwardSlashes = backup.split(/\//).length;
		const nBackwardSlashes = backup.split(/\\/).length;

		if (nForwardSlashes > nBackwardSlashes) {
			this.backup(backup.replace(/\//g, "\\"));
		} else {
			this.backup(backup.replace(/\\/g, "/"));
		}
	};

	Restore.prototype.schemaFromCode = function() {
		this.parseSchemaVisible(true);
	};

	Restore.prototype.tablesFromCode = function() {
		this.parseTablesVisible(true);
	};

	Restore.prototype.parseTables = function(text) {
		const tables = new Set();

		for (const re of EXTRACT_TABLES_RES) {
			const result = text.match(re);

			if (result !== null) {
				result
					.map(function(item) {
						return re.exec(item);
					})
					.filter(function(item) {
						return item !== null;
					})
					.forEach(function(item) {
						tables.add(item[1]);
					});
			}
		}

		return Array.from(tables);
	};

	Restore.prototype.parseSchema = function(text) {
		const tables = this.parseTables(text);
		const schema = new Set();

		tables.forEach(function(tableName) {
			const index = tableName.indexOf(".");

			if (index > 0) {
				schema.add(tableName.substring(0, index));
			}
		});

		return Array.from(schema);
	};

	return Restore;
});
