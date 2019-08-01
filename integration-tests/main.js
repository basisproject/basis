"use strict";

const Promise = require('bluebird');
const rp = require('request-promise');
const Exonum = require('exonum-client');
const protobuf = require('protobufjs');
const uuid = require('uuid/v4');
const Cmd = require('command-line-args');

const cmd_args = [
	{name: 'command', defaultOption: true},
	{name: 'key', alias: 'k'},
	{name: 'data', alias: 'd'},
];

function parse_cmd() {
	const args = Cmd(cmd_args);
	args.data = JSON.parse(args.data || '{}');
	return args;
}

const api_url = 'http://127.0.0.1:13007/api';
const keypairs = {
	admin: {
		pub: 'e59e03bccb4377d01d411dd71e7c1eccbaf71ce0b6d9c2c7d1f9d515aaa25248',
		sec: '91e467f3e875dd35011a5f2efc49c6660a57fa5c5fffded95ecd9515ae9b9704e59e03bccb4377d01d411dd71e7c1eccbaf71ce0b6d9c2c7d1f9d515aaa25248',
	},
	jerry: {
		pub: '527027663d24c47a366d026df1deacea69448b59b8abfc85cb6d8465f21e10a9',
		sec: 'c4ff7549d59c7d98a61b7c0da83df48efc8624f34c94170b67f5b5243cb8917a527027663d24c47a366d026df1deacea69448b59b8abfc85cb6d8465f21e10a9',
	},
};
var keypair;

const Timestamp = {
	type: (function() {
		const Type = new protobuf.Type('Timestamp');
		Type.add(new protobuf.Field('seconds', 1, 'int64'));
		Type.add(new protobuf.Field('nanos', 2, 'int32'));
		return Type;
	})(),
	gen: function() {
		const now = new Date().getTime();
		const seconds = Math.floor(now / 1000);
		return {
			seconds: Math.floor(now / 1000),
			nanos: (now - (seconds * 1000)) * 1000000,
		};
	},
};

const Pubkey = {
	type: (function() {
		const Type = new protobuf.Type('Pubkey');
		Type.add(new protobuf.Field('data', 1, 'bytes'));
		return Type;
	})(),
	gen: function(pubkey) {
		return {data: Exonum.hexadecimalToUint8Array(pubkey)};
	},
};

const CompanyType = {
	map: {
		UNKNOWN: 0,
		PUBLIC: 1,
		MEMBER: 2,
		PRIVATE: 3,
	},
	type: new protobuf.Enum('CompanyType', this.map),
	gen: function(val) {
		return CompanyType.map[val.toUpperCase()] || 0;
	},
};

const actions = {
	user_create: function(data) {
		const Transaction = new protobuf.Type('CustomMessage');
		Transaction.add(Pubkey.type);
		Transaction.add(Timestamp.type);
		Transaction.add(new protobuf.Field('id', 1, 'string'));
		Transaction.add(new protobuf.Field('pubkey', 2, 'Pubkey'));
		Transaction.add(new protobuf.Field('roles', 3, 'string', 'repeated'));
		Transaction.add(new protobuf.Field('email', 4, 'string'));
		Transaction.add(new protobuf.Field('name', 5, 'string'));
		Transaction.add(new protobuf.Field('meta', 6, 'string'));
		Transaction.add(new protobuf.Field('created', 7, 'Timestamp'));
		const trans = Exonum.newTransaction({
			author: keypair.pub,
			service_id: 128,
			message_id: 0,
			schema: Transaction,
		});
		if(!data.id) data.id = uuid();
		data.pubkey = Pubkey.gen(data.pubkey);
		if(data.meta) data.meta = JSON.stringify(data.meta);
		data.created = Timestamp.gen();
		return Promise.resolve(trans.send(api_url+'/explorer/v1/transactions', data, keypair.sec));
	},

	user_update: function(data) {
		const Transaction = new protobuf.Type('CustomMessage');
		Transaction.add(Timestamp.type);
		Transaction.add(new protobuf.Field('id', 1, 'string'));
		Transaction.add(new protobuf.Field('email', 2, 'string'));
		Transaction.add(new protobuf.Field('name', 3, 'string'));
		Transaction.add(new protobuf.Field('meta', 4, 'string'));
		Transaction.add(new protobuf.Field('updated', 5, 'Timestamp'));
		const trans = Exonum.newTransaction({
			author: keypair.pub,
			service_id: 128,
			message_id: 1,
			schema: Transaction,
		});
		if(data.meta) data.meta = JSON.stringify(data.meta);
		data.updated = Timestamp.gen();
		return Promise.resolve(trans.send(api_url+'/explorer/v1/transactions', data, keypair.sec));
	},

	user_set_pubkey: function(data) {
		const Transaction = new protobuf.Type('CustomMessage');
		Transaction.add(Pubkey.type);
		Transaction.add(Timestamp.type);
		Transaction.add(new protobuf.Field('id', 1, 'string'));
		Transaction.add(new protobuf.Field('pubkey', 2, 'Pubkey'));
		Transaction.add(new protobuf.Field('memo', 3, 'string'));
		Transaction.add(new protobuf.Field('updated', 4, 'Timestamp'));
		const trans = Exonum.newTransaction({
			author: keypair.pub,
			service_id: 128,
			message_id: 2,
			schema: Transaction,
		});
		data.pubkey = Pubkey.gen(data.pubkey);
		data.updated = Timestamp.gen();
		return Promise.resolve(trans.send(api_url+'/explorer/v1/transactions', data, keypair.sec));
	},

	user_set_roles: function(data) {
		const Transaction = new protobuf.Type('CustomMessage');
		Transaction.add(Timestamp.type);
		Transaction.add(new protobuf.Field('id', 1, 'string'));
		Transaction.add(new protobuf.Field('roles', 2, 'string', 'repeated'));
		Transaction.add(new protobuf.Field('memo', 3, 'string'));
		Transaction.add(new protobuf.Field('updated', 4, 'Timestamp'));
		const trans = Exonum.newTransaction({
			author: keypair.pub,
			service_id: 128,
			message_id: 3,
			schema: Transaction,
		});
		data.updated = Timestamp.gen();
		return Promise.resolve(trans.send(api_url+'/explorer/v1/transactions', data, keypair.sec));
	},

	user_delete: function(data) {
		const Transaction = new protobuf.Type('CustomMessage');
		Transaction.add(new protobuf.Field('id', 1, 'string'));
		Transaction.add(new protobuf.Field('memo', 2, 'string'));
		const trans = Exonum.newTransaction({
			author: keypair.pub,
			service_id: 128,
			message_id: 4,
			schema: Transaction,
		});
		return Promise.resolve(trans.send(api_url+'/explorer/v1/transactions', data, keypair.sec));
	},

	users_get: function(data) {
		return rp({
			url: `${api_url}/services/factor/v1/users`,
			json: true,
			qs: data,
		});
	},

	user_get: function(data) {
		return rp({
			url: `${api_url}/services/factor/v1/users/info`,
			json: true,
			qs: data,
		});
	},

	company_create_private: function(data) {
		const Transaction = new protobuf.Type('CustomMessage');
		Transaction.add(Timestamp.type);
		Transaction.add(new protobuf.Field('id', 1, 'string'));
		Transaction.add(new protobuf.Field('email', 2, 'string'));
		Transaction.add(new protobuf.Field('name', 3, 'string'));
		Transaction.add(new protobuf.Field('created', 4, 'Timestamp'));
		const trans = Exonum.newTransaction({
			author: keypair.pub,
			service_id: 128,
			message_id: 5,
			schema: Transaction,
		});
		if(!data.id) data.id = uuid();
		data.created = Timestamp.gen();
		return Promise.resolve(trans.send(api_url+'/explorer/v1/transactions', data, keypair.sec));
	},

	company_update: function(data) {
		const Transaction = new protobuf.Type('CustomMessage');
		Transaction.add(Timestamp.type);
		Transaction.add(new protobuf.Field('id', 1, 'string'));
		Transaction.add(new protobuf.Field('email', 2, 'string'));
		Transaction.add(new protobuf.Field('name', 3, 'string'));
		Transaction.add(new protobuf.Field('updated', 4, 'Timestamp'));
		const trans = Exonum.newTransaction({
			author: keypair.pub,
			service_id: 128,
			message_id: 6,
			schema: Transaction,
		});
		data.updated = Timestamp.gen();
		return Promise.resolve(trans.send(api_url+'/explorer/v1/transactions', data, keypair.sec));
	},

	company_set_type: function(data) {
		const Transaction = new protobuf.Type('CustomMessage');
		Transaction.add(CompanyType.type);
		Transaction.add(Timestamp.type);
		Transaction.add(new protobuf.Field('id', 1, 'string'));
		Transaction.add(new protobuf.Field('ty', 2, 'CompanyType'));
		Transaction.add(new protobuf.Field('updated', 3, 'Timestamp'));
		const trans = Exonum.newTransaction({
			author: keypair.pub,
			service_id: 128,
			message_id: 7,
			schema: Transaction,
		});
		data.updated = Timestamp.gen();
		data.ty = CompanyType.gen(data.ty);
		return Promise.resolve(trans.send(api_url+'/explorer/v1/transactions', data, keypair.sec));
	},

	company_delete: function(data) {
		const Transaction = new protobuf.Type('CustomMessage');
		Transaction.add(new protobuf.Field('id', 1, 'string'));
		Transaction.add(new protobuf.Field('memo', 2, 'string'));
		const trans = Exonum.newTransaction({
			author: keypair.pub,
			service_id: 128,
			message_id: 8,
			schema: Transaction,
		});
		return Promise.resolve(trans.send(api_url+'/explorer/v1/transactions', data, keypair.sec));
	},

	keypair: function() {
		return Promise.resolve(Exonum.keyPair());
	},
};

function main() {
	const args = parse_cmd();
	const action = actions[args.command];
	if(!action) {
		console.log('Missing command `'+args.command+'`');
		console.log('Usage: node '+process.argv[1]+' <command> [options]');
		process.exit(1);
	}
	switch(args.key) {
		case 'random':
		case 'rand':
			keypair = Exonum.keyPair();
			break;
		default:
			keypair = keypairs[args.key || 'admin'];
			break;
	}

	return action(args.data)
		.then(function(res) {
			console.log('---\n'+JSON.stringify(res, null, 2));
		});
}

main()
	.catch(function(err) { console.log('err: ', err, err.stack); })
	.finally(function() { setTimeout(process.exit, 100); });

