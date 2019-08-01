const Promise = require('bluebird');
const uuid = require('uuid/v4');
const Exonum = require('exonum-client');
const trans = require('../helpers/transactions');
const tx = trans.types;
const bootstrap = require('../helpers/bootstrap');
const config = require('../helpers/config');
const Users = require('../helpers/users');

describe('users', function() {
	jasmine.DEFAULT_TIMEOUT_INTERVAL = 30000;

	beforeAll((done) => {
		bootstrap.load().then(done).catch(done.fail);
	});

	afterAll((done) => {
		trans.clear_users();
		bootstrap.unload().then(done).catch(done.fail);
	});

	const jerry_user_id = uuid();
	const {publicKey: jerry_pubkey, secretKey: jerry_seckey} = Exonum.keyPair();
	const jerry_email = 'jerry@thatscool.net';
	const jerry_email_new = 'jerry2@jerrythejerjer.net';

	const sandra_user_id = uuid();
	const {publicKey: sandra_pubkey, secretKey: sandra_seckey} = Exonum.keyPair();
	const sandra_email = 'sandra@thatscool.net';

	trans.add_user('root', config.bootstrap_user.pub, config.bootstrap_user.sec);
	trans.add_user('jerry', jerry_pubkey, jerry_seckey);
	trans.add_user('sandra', sandra_pubkey, sandra_seckey);

	it('can be created', async () => {
		// create jerry
		let jerry = await Users.get({id: jerry_user_id});
		expect(jerry).toBe(null);
		var txid = await trans.send_as('root', tx.user.TxCreate, {
			id: jerry_user_id,
			pubkey: jerry_pubkey,
			roles: ['User'],
			email: jerry_email,
			name: 'jerry anderson',
			meta: '{}',
			created: new Date().toISOString(),
		});
		await Promise.delay(200);
		var status = await trans.status(txid);
		expect(status.success).toBe(true);
		jerry = await Users.get({id: jerry_user_id});
		expect(jerry.id).toBe(jerry_user_id);

		// create sandra
		let sandra = await Users.get({id: sandra_user_id});
		expect(sandra).toBe(null);
		var txid = await trans.send_as('root', tx.user.TxCreate, {
			id: sandra_user_id,
			pubkey: sandra_pubkey,
			roles: ['User'],
			email: sandra_email,
			name: 'sandra sanderton',
			meta: '{}',
			created: new Date().toISOString(),
		});
		await Promise.delay(200);
		var status = await trans.status(txid);
		expect(status.success).toBe(true);
		sandra = await Users.get({id: sandra_user_id});
		expect(sandra.id).toBe(sandra_user_id);
	});

	it('can update themselves', async () => {
		var txid = await trans.send_as('jerry', tx.user.TxUpdate, {
			id: jerry_user_id,
			email: jerry_email_new,
			name: 'jerry *THE JERJER* anderson',
			meta: '{"friends":99}',
			updated: new Date().toISOString(),
		});
		await Promise.delay(200);
		var status = await trans.status(txid);
		expect(status.success).toBe(true);
		var jerry = await Users.get({id: jerry_user_id});
		expect(jerry.email).toBe(jerry_email_new);
	})

	it('stops users from editing other users', async () => {
		// sandra cannot update jerjer
		var data2 = {
			id: jerry_user_id,
			email: 'jerry.sux@jerry.is.NOT.cool.net',
			name: 'jerry *THE JERKJERK* JERKSTON',
			meta: '{"friends":0}',
			updated: new Date().toISOString(),
		};
		var txid = await trans.send_as('sandra', tx.user.TxUpdate, data2);
		await Promise.delay(200);
		var status = await trans.status(txid);
		expect(status.success).toBe(false);
		expect(status.committed).toBe(true);
		var jerry = await Users.get({id: jerry_user_id});
		expect(jerry.email).toBe(jerry_email_new);
		expect(jerry.email).not.toBe(data2.email);
	});

	it('can have their permissions changed by admin', async () => {
		var txid = await trans.send_as('root', tx.user.TxSetRoles, {
			id: sandra_user_id,
			roles: ['IdentityAdmin'],
			memo: 'great job, sandra. you\'ve earned this',
			updated: new Date().toISOString(),
		});
		await Promise.delay(200);
		var status = await trans.status(txid);
		expect(status.success).toBe(true);
	});

	it('lets users with correct permissions edit other users', async () => {
		// sandra CAN NOW abuse her power and update jerjer the jerkjerk
		var txid = await trans.send_as('sandra', tx.user.TxUpdate, {
			id: jerry_user_id,
			email: 'jerry.sux@jerry.is.NOT.cool.net',
			name: 'jerry *THE JERKJERK* JERKSTON',
			meta: '{"friends":0}',
			updated: new Date().toISOString(),
		});
		await Promise.delay(200);
		var status = await trans.status(txid);
		expect(status.success).toBe(true);
		var jerry = await Users.get({id: jerry_user_id});
		expect(jerry.email).not.toBe(jerry_email_new);
	});

	it('can have their pubkey changed by an admin', async () => {
		//var txid = await trans.send_as('root', 'factor.
	});

	it('can be deleted', async () => {
		var data = {
			id: jerry_user_id,
			memo: `just gettin deleted, huh? thats cool...`,
			deleted: new Date().toISOString(),
		};
		var params = {
			pubkey: jerry_pubkey,
			privkey: jerry_seckey,
			message_id: 4,
		};
		await Promise.delay(100);
		var user = await Users.get({id: jerry_user_id});
		expect(user.id).toBe(jerry_user_id);
		var txid = await trans.send(tx.user.TxDelete, data, params);
		await Promise.delay(100);
		var status = await trans.status(txid);
		expect(status.success).toBe(true);
		user = await Users.get({id: jerry_user_id});
		expect(user).toBe(null);

		var data = {
			id: sandra_user_id,
			memo: `just gettin deleted, huh? thats cool...`,
			deleted: new Date().toISOString(),
		};
		var params = {
			pubkey: sandra_pubkey,
			privkey: sandra_seckey,
			message_id: 4,
		};
		await Promise.delay(100);
		var user = await Users.get({email: sandra_email});
		expect(user.id).toBe(sandra_user_id);
		var txid = await trans.send(tx.user.TxDelete, data, params);
		await Promise.delay(100);
		var status = await trans.status(txid);
		expect(status.success).toBe(true);
		user = await Users.get({id: sandra_user_id});
		expect(user).toBe(null);
	});
});

