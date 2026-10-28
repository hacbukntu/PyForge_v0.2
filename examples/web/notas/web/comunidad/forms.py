"""Formularios acotados; nunca aceptan el propietario desde el navegador."""
from django import forms

class NoteForm(forms.Form):
    text = forms.CharField(label='Nota privada', max_length=500, widget=forms.Textarea)

class ProductForm(forms.Form):
    name = forms.CharField(label='Producto', max_length=120)
    units = forms.IntegerField(label='Unidades', min_value=0, max_value=1000000)

class AppointmentForm(forms.Form):
    title = forms.CharField(label='Cita', max_length=120)
    date = forms.DateField(label='Fecha', widget=forms.DateInput(attrs={'type':'date'}))
