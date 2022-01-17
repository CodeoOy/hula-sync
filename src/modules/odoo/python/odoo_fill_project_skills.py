import sys 
import json
from datetime import datetime, date
from typing import Optional, List
import jsonpickle
import ahocorasick

url = str(sys.argv[1])
db = str(sys.argv[2])
username = str(sys.argv[3])
password = str(sys.argv[4])
hula_skills = json.loads(str(sys.argv[5]))

import xmlrpc.client

common = xmlrpc.client.ServerProxy('{}/xmlrpc/2/common'.format(url))
uid = common.authenticate(db, username, password, {})
models = xmlrpc.client.ServerProxy('{}/xmlrpc/2/object'.format(url))

class MergedSkill(object):
    def __init__(self, hula_id: str, odoo_id: str, label: str, aliases: Optional[List[str]]) -> None:
        self.hula_id = hula_id
        self.odoo_id = odoo_id
        self.label = label
        self.aliases = aliases

    hula_id: str
    odoo_id: str
    label_id: str
    aliases = Optional[List[str]]


def find_skills(texts, merged_skills: List[MergedSkill]):
    if not merged_skills:
        return []

    A = ahocorasick.Automaton()

    for skill in merged_skills:
        A.add_word(skill.label.lower(), skill.odoo_id)
        if skill.aliases:
            for alias in skill.aliases:
                A.add_word(alias.lower(), skill.odoo_id)

    A.make_automaton()

    filtered_texts = [t.lower() for t in texts if t]
    whole_text = ' '.join(filtered_texts)

    # search
    found_items = A.iter_long(whole_text)
    found_skill_ids = map(lambda x: x[1], found_items)

    # remove duplicates while preserving the order
    # Note: set not used because it does not preserve the order
    return list(dict.fromkeys(found_skill_ids).keys())


def update_lead(id, found_skill_ids=[], project_name=None, begin_date=None):
    found_skills_count = len(found_skill_ids)
    updated_fields = {}

    for idx in range(4):
        # Note: 'None' value is not accepted by Odoo by default, False used instead
        skill_id = found_skill_ids[idx] if idx < found_skills_count else False
        updated_fields[f'x_studio_skill_{idx+1}'] = skill_id

    if project_name is not None:
        updated_fields['x_studio_project_name'] = project_name

    if begin_date is not None:
        updated_fields['x_studio_begin'] = begin_date

    models.execute_kw(db, uid, password, 'crm.lead', 'write', [[id],
            {
                'x_studio_disable_skill_generation': True,
                **updated_fields
            }])


def get_mails(ids):
    return (models.execute_kw(db, uid, password, 'mail.message', 'read', [ids], {'fields': ['id', 'date', 'subject', 'body']})
        if ids
        else [])


def get_mail_texts(mails):
    texts = []
    for mail in mails:
        texts.append(mail['subject'])
        texts.append(mail['body'])
    return texts


def generate_begin_date(lead):
    if lead['x_studio_begin']:
        return None

    return date.today().isoformat()


def generate_project_name(lead):
    if lead['x_studio_project_name']:
        return None

    subject = lead['name']
    date = datetime.today().strftime('%-d.%-m.%Y')
    return f"{subject} {date}"


def merge_skills(hula_skills, odoo_skills):
    hula_dict = {s['label'].lower(): s for s in hula_skills}
    odoo_dict = {s['x_name'].lower(): s for s in odoo_skills}

    merged_skills = []
    for label in odoo_dict.keys():
        odoo_skill = odoo_dict[label]
        hula_skill = hula_dict.get(label, None)
        if not hula_skill:
            continue

        merged_skills.append(MergedSkill(
            hula_id=hula_skill['id'],
            odoo_id=odoo_skill['id'],
            label=label,
            aliases=hula_skill.get('aliases'))
        )

    return merged_skills


def main():
    leads = models.execute_kw(db, uid, password,
        'crm.lead', 'search_read',
        [[['x_studio_disable_skill_generation', '!=', True]]],
        { 'fields': [
            'id', 
            'name',
            'description',
            'x_studio_begin',
            'x_studio_project_name', 
            'x_studio_description',
            'message_ids'
        ]})


    odoo_skills = models.execute_kw(db, uid, password,
        'x_hula_skill', 'search_read', [], {'fields': ['id', 'display_name', 'x_name']})

    merged_skills = merge_skills(hula_skills, odoo_skills)

    for lead in leads:
        mails = get_mails(lead['message_ids'])
        mail_texts = get_mail_texts(mails)
        # Note: order is significant: first search skill labels
        # from titles and descriptions, then from message contents. 
        texts = [
            lead['name'],
            lead['description'],
            lead['x_studio_project_name'],
            lead['x_studio_description'],
            *mail_texts]

        found_skills = find_skills(texts, merged_skills)

        project_name = generate_project_name(lead)
        begin_date = generate_begin_date(lead)
        update_lead(lead['id'], found_skills, project_name=project_name, begin_date=begin_date)

    lead_names = [x['name'] for x in leads]
    print(jsonpickle.encode(lead_names, unpicklable=False))


main()