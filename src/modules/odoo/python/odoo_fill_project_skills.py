import sys
import jsonpickle
import ahocorasick

url = str(sys.argv[1])
db = str(sys.argv[2])
username = str(sys.argv[3])
password = str(sys.argv[4])

import xmlrpc.client

common = xmlrpc.client.ServerProxy('{}/xmlrpc/2/common'.format(url))
uid = common.authenticate(db, username, password, {})
models = xmlrpc.client.ServerProxy('{}/xmlrpc/2/object'.format(url))


def find_skills(texts, skills):
    A = ahocorasick.Automaton()

    for skill in skills:
        skill_label = skill['x_name'].lower()
        skill_id = skill['id']
        A.add_word(skill_label, skill_id)

    A.make_automaton()

    filtered_texts = [t.lower() for t in texts if t]
    whole_text = ' '.join(filtered_texts)

    # search
    found_items = A.iter_long(whole_text)
    found_skill_ids = map(lambda x: x[1], found_items)

    # remove duplicates while preserving the order
    # Note: set not used because it does not preserve the order
    return list(dict.fromkeys(found_skill_ids).keys())


def update_lead(id, found_skill_ids):
    found_skills_count = len(found_skill_ids)
    skill_fields = {}

    for idx in range(4):
        # Note: 'None' value is not accepted by Odoo by default, False used instead
        skill_id = found_skill_ids[idx] if idx < found_skills_count else False
        skill_fields[f'x_studio_skill_{idx+1}'] = skill_id        

    models.execute_kw(db, uid, password, 'crm.lead', 'write', [[id],
            {
                'x_studio_disable_skill_generation': True,
                **skill_fields
            }])


def get_mail_texts(ids):
    mails = (models.execute_kw(db, uid, password, 'mail.message', 'read', [ids], {'fields': ['id', 'subject', 'body']})
        if ids
        else [])
    texts = []
    for mail in mails:
        texts.append(mail['subject'])
        texts.append(mail['body'])
    return texts


def main():
    leads = models.execute_kw(db, uid, password,
        'crm.lead', 'search_read',
        [[['x_studio_disable_skill_generation', '!=', True]]],
        # [[]],
        { 'fields': [
            'id', 
            'name',
            'description',
            'x_studio_project_name', 
            'x_studio_description',
            'message_ids'
        ]})


    skills = models.execute_kw(db, uid, password,
        'x_hula_skill', 'search_read', [], {'fields': ['id', 'display_name', 'x_name']})

    for lead in leads:
        mail_texts = get_mail_texts(lead['message_ids'])
        # Note: order is significant: first search skill labels
        # from titles and descriptions, then from message contents. 
        texts = [
            lead['name'],
            lead['description'],
            lead['x_studio_project_name'],
            lead['x_studio_description'],
            *mail_texts]

        found_skills = find_skills(texts, skills)
        update_lead(lead['id'], found_skills)

    lead_names = [x['name'] for x in leads]
    print(jsonpickle.encode(lead_names, unpicklable=False))


main()