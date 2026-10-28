"""Formularios acotados; nunca aceptan el propietario desde el navegador."""
from django import forms


class CaptureForm(forms.Form):
    title = forms.CharField(label='Título', max_length=120)
    kind = forms.ChoiceField(
        label='Tipo',
        choices=[('general', 'General'), ('lead', 'Lead'), ('producto', 'Producto'), ('idea', 'Idea')],
    )
    content = forms.CharField(label='Contenido', max_length=2000, widget=forms.Textarea)
