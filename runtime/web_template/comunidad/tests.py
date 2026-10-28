"""Regresiones de privacidad y contrato HTTP; el motor nativo se prueba también de extremo a extremo."""
from unittest.mock import patch
from django.test import TestCase, Client
from .models import SavedEntry

class WebTests(TestCase):
    """Cada caso usa una base efímera y sesiones independientes."""
    def test_anonymous_sessions_are_private(self):
        a, b = Client(), Client()
        response = a.post('/notas/', {'title':'Privado','kind':'general','content':'DATO-PRUEBA'})
        self.assertEqual(response.status_code,200)
        self.assertEqual(SavedEntry.objects.count(),1)
        self.assertContains(a.get('/notas/'),'DATO-PRUEBA')
        self.assertNotContains(b.get('/notas/'),'DATO-PRUEBA')

    def test_csrf_and_no_sources(self):
        strict=Client(enforce_csrf_checks=True)
        self.assertEqual(strict.post('/notas/',{}).status_code,403)
        self.assertEqual(strict.post('/api/snake/',{},content_type='application/json').status_code,403)
        self.assertEqual(self.client.get('/api/snake/').status_code,405)
        for path in ['/.env','/settings.py','/data/sitio.sqlite3']:
            self.assertEqual(self.client.get(path).status_code,404)

    def test_game_external_script_and_server_owned_state(self):
        response=self.client.get('/juego/')
        self.assertContains(response,'/static/snake.js')
        self.assertNotContains(response,'<script>')
        self.assertIn("script-src 'self'",response['Content-Security-Policy'])
        state={'cuerpo':[208,207,206],'cantidad':3,'comida':212,'direccion':0,'puntos':0,'fin':False}
        with patch('comunidad.views.run_st_json',return_value=state) as native:
            self.assertEqual(self.client.post('/api/snake/',{'accion':'reiniciar'},content_type='application/json').status_code,200)
            self.client.post('/api/snake/',{'accion':'paso','direccion':1,'estado':{'puntos':999}},content_type='application/json')
            self.assertEqual(native.call_args.args[0]['estado']['puntos'],0)
            self.assertEqual(native.call_args.args[0]['direccion'],1)
        self.assertEqual(Client().post('/api/snake/',{'accion':'paso'},content_type='application/json').status_code,409)

    def test_saved_text_is_escaped(self):
        self.client.post('/notas/',{'title':'<script>','kind':'general','content':"'; DROP TABLE comunidad_savedentry; --"})
        self.assertEqual(SavedEntry.objects.count(),1)
        self.assertContains(self.client.get('/notas/'),'&lt;script&gt;')
