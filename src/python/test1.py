url1 = 'https://giraffe-code.odoo.com'
db1 = 'giraffe-code'
username1 = 'sintopek@gmail.com'
password1 = '30ae863f-f180-483c-8a70-934e5719b889'


import xmlrpc.client
info = xmlrpc.client.ServerProxy('https://demo.odoo.com/start').start()
url, db, username, password = \
    info['host'], info['database'], info['user'], info['password']

common = xmlrpc.client.ServerProxy('{}/xmlrpc/2/common'.format(url))
v = common.version()

print("----")

uid = common.authenticate(db, username, password, {})

print(uid)

models = xmlrpc.client.ServerProxy('{}/xmlrpc/2/object'.format(url))
a = models.execute_kw(db, uid, password,
    'res.partner', 'check_access_rights',
    ['read'], {'raise_exception': False})

print(a)





