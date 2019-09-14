"use strict";

const Exonum = require('exonum-client');
const rp = require('request-promise');
const protobuf = require('./protobuf');
const config = require('./config');

const types = {};
const message_id_map = (function() {
	// these MUST be ordered in the same way as basis/src/block/transactions/mod.rs
	const transactions = [
		'user.TxCreate',
		'user.TxUpdate',
		'user.TxSetPubkey',
		'user.TxSetRoles',
		'user.TxDelete',

		'company.TxCreatePrivate',
		'company.TxUpdate',
		'company.TxSetType',
		'company.TxDelete',

		'company_member.TxCreate',
		'company_member.TxSetRoles',
		'company_member.TxDelete',

		'product.TxCreate',
		'product.TxUpdate',
		'product.TxDelete',

		'order.TxCreate',
		'order.TxUpdateStatus',
	];
	const map = {};
	let i = 0;
	transactions.forEach((t) => { map[t] = i++; });
	return map;
})();

Object.keys(message_id_map).forEach((key) => {
	const [type, tx] = key.split('.');
	if(!types[type]) types[type] = {};
	types[type][tx] = {
		type: `basis.${key}`,
		msg_id: message_id_map[key],
	}
});

exports.types = types;

exports.make = (type, data, params) => {
	const Transaction = protobuf.root.lookupType(type.type);
	const map_types = {
		'google.protobuf.Timestamp': 'Timestamp',
		'exonum.Hash': 'Hash',
		'exonum.PublicKey': 'Pubkey',
		'CompanyType': 'CompanyType',
		'Product.Unit': 'Unit',
		'Product.Effort.Time': 'Time',
		'Order.ProcessStatus': 'ProcessStatus',
	};
	Object.keys(Transaction.fields).forEach((field) => {
		if(typeof(data[field]) == 'undefined') return;
		const spec = Transaction.fields[field];
		const mapping = map_types[spec.type];
		if(!mapping) return;
		data[field] = protobuf.types[mapping].gen(data[field]);
	});
	const errmsg = Transaction.verify(data);
	if(errmsg) {
		throw new Error(`transaction: ${type}: verification error: ${errmsg}`);
	}
	const trans = Exonum.newTransaction({
		schema: Transaction,
		author: params.pubkey,
		service_id: params.service_id || config.service_id,
		message_id: params.message_id || type.msg_id,
	});
	return trans;
};

exports.send = async (type, data, params) => {
	const trans = exports.make(type, data, params);
	return trans.send(`${config.endpoint}/explorer/v1/transactions`, data, params.privkey);
};

const whoswho = {};
exports.add_user = (who, pubkey, seckey) => {
	whoswho[who] = {pub: pubkey, sec: seckey};
};

exports.clear_users = () => {
	Object.keys(whoswho).forEach((key) => delete whoswho[key]);
};

exports.send_as = async (who, type, data, params, options) => {
	options || (options = {});
	const user = whoswho[who];
	if(!user) throw new Error(`helpers/transactions::send_as() -- missing user ${who}`);
	const newparams = Object.assign({}, {
		pubkey: user.pub,
		privkey: user.sec,
	}, params || {});
	let txid = await exports.send(type, data, newparams);
	if(options.no_wait) {
		return txid;
	}
	let status = exports.wait(txid, options);
	status.txid = txid;
	return status;
};

exports.get = async (txid) => {
	const res = await rp({
		url: `${config.endpoint}/explorer/v1/transactions?hash=${txid}`,
		json: true,
	});
	return res;
};

function extract_status(trans) {
	return {
		committed: trans.type == 'committed',
		success: trans.status.type == 'success',
		code: trans.status.code,
		description: trans.status.description,
	}
}

exports.wait = async (txid, options) => {
	options || (options = {});
	let timeout = false;
	setTimeout(() => timeout = true, options.timeout || 10000);
	let trans = null;
	while(true) {
		const res = await rp({
			url: `${config.endpoint}/explorer/v1/transactions?hash=${txid}`,
			json: true,
		});
		if(timeout) {
			if(options.raw) {
				throw new Error(`helpers/transactions::wait() -- timeout`);
			}
			return {missing: true};
		}
		if(res && res.type != 'unknown') {
			trans = res;
			break;
		}
	}
	if(options.raw) return trans;
	return extract_status(trans);
};

exports.status = async (txid) => {
	const res = await exports.get(txid);
	if(!res || res.type == 'unknown') return {missing: true};
	return extract_status(res);
};

