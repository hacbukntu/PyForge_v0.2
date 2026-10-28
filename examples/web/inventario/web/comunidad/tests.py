"""Pruebas de los tres modelos de negocio, separados del foro real."""
from django.contrib.auth import get_user_model
from django.test import TestCase, Client
from .models import Note, Product, Appointment

class ExamplesTests(TestCase):
    """Las cuentas y bases SQL solo existen durante cada prueba."""
    def setUp(self):
        self.user = get_user_model().objects.create_user('ana', password='Prueba-larga!1357')
        self.other = get_user_model().objects.create_user('otro', password='Prueba-larga!2468')
        self.client.force_login(self.user)

    def test_three_models_and_owner(self):
        for route, model, data in [('/notas/',Note,{'text':'Nota'}),('/inventario/',Product,{'name':'Teclado','units':4}),('/agenda/',Appointment,{'title':'Reunión','date':'2026-10-15'})]:
            self.assertEqual(self.client.post(route, data).status_code,302)
            self.assertEqual(model.objects.get().owner,self.user)
        Product.objects.create(owner=self.other,name='PRODUCTO-AJENO',units=9)
        self.assertNotContains(self.client.get('/inventario/'),'PRODUCTO-AJENO')

    def test_validation_and_escape(self):
        self.client.post('/inventario/',{'name':'Inválido','units':-1})
        self.assertEqual(Product.objects.count(),0)
        self.client.post('/notas/',{'text':'<script>alert(1)</script>'})
        self.assertContains(self.client.get('/notas/'),'&lt;script&gt;')
        self.client.post('/notas/',{'text':"'; DROP TABLE comunidad_note; --"})
        self.assertEqual(Note.objects.count(),2)

    def test_csrf_and_no_forum(self):
        strict=Client(enforce_csrf_checks=True)
        strict.force_login(self.user)
        self.assertEqual(strict.post('/inventario/',{'name':'X','units':1}).status_code,403)
        self.assertEqual(self.client.get('/foro/').status_code,404)
        for route in ['/settings.py','/.env','/data/sitio.sqlite3']:
            self.assertEqual(self.client.get(route).status_code,404)
