"""Valida formularios e imágenes antes de guardar; no conserva nombre ni EXIF originales."""
import io
import warnings
from django import forms
from PIL import Image, UnidentifiedImageError


class TopicForm(forms.Form):
    title = forms.CharField(label='Título', max_length=140)
    body = forms.CharField(label='Error o pregunta', max_length=8000, widget=forms.Textarea)
    screenshot = forms.FileField(label='Captura PNG o JPEG (opcional, máximo 2 MB)', required=False)

    def clean_screenshot(self):
        """Decodifica y recodifica PNG/JPEG; rechaza SVG, archivos falsos y bombas de imagen."""
        upload = self.cleaned_data.get('screenshot')
        if not upload:
            return b''
        if upload.size > 2 * 1024 * 1024:
            raise forms.ValidationError('La captura supera 2 MB.')
        try:
            with warnings.catch_warnings():
                warnings.simplefilter('error', Image.DecompressionBombWarning)
                with Image.open(upload) as image:
                    if image.format not in {'PNG','JPEG'} or image.width * image.height > 4_000_000:
                        raise ValueError('Formato o dimensiones no permitidos')
                    image.load()
                    clean = image.convert('RGB')
                    clean.thumbnail((1600,1600))
                    clean.info.clear()  # Quita EXIF y metadatos que convert() puede conservar.
                    output = io.BytesIO()
                    clean.save(output, format='PNG')
                    data = output.getvalue()
                    if len(data) > 2 * 1024 * 1024:
                        raise ValueError('Captura recodificada demasiado grande')
                    return data
        except (OSError, ValueError, UnidentifiedImageError, Image.DecompressionBombError, Image.DecompressionBombWarning):
            raise forms.ValidationError('Usa una captura PNG o JPEG válida, de hasta cuatro megapíxeles.') from None


class ReplyForm(forms.Form):
    body = forms.CharField(label='Respuesta', max_length=4000, widget=forms.Textarea)


class NoteForm(forms.Form):
    text = forms.CharField(label='Nota privada', max_length=500)
