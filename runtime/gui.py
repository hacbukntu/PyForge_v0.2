"""Proceso Qt opcional: interfaz en su hilo principal, protocolo local sin eval.

El runtime inicia este código solo con permiso gui. Las operaciones ST de archivos
u otras bibliotecas ocurren en el auxiliar padre y no bloquean el hilo de Qt.
"""
import json
import os
import sys
from PySide6 import QtCore, QtWidgets


# stdout transporta exclusivamente eventos JSON: no imprimir diagnósticos aquí.
# Si el lector ST se retrasa, emit avisa de pérdida sin congelar la ventana.
DROPPED = False
os.set_blocking(1, False)


def emit(event):
    """Envía eventos pequeños; nunca incluye contenido de campos de contraseña."""
    global DROPPED
    if DROPPED:
        event = {'evento': 'error', 'codigo': 'GUI_EVENTOS_DESCARTADOS'}
    raw = json.dumps(event, ensure_ascii=True).encode('ascii') + b'\n'
    if len(raw) <= 4000:
        try:
            os.write(1, raw)
            DROPPED = False
        except BlockingIOError:
            DROPPED = True  # no bloquea Qt aunque ST deje de consultar eventos
        except BrokenPipeError:
            QtWidgets.QApplication.quit()


def main():
    """Construye una ventana declarativa y atiende cambios mediante QSocketNotifier."""
    # Primera línea: diseño enviado por Runtime.gui_create; luego llegan órdenes.
    spec = json.loads(sys.stdin.buffer.readline(4096))
    application = QtWidgets.QApplication([])
    window = QtWidgets.QWidget()
    window.setWindowTitle(spec.get('titulo', 'Aplicación ST'))
    # Personaliza aquí el tamaño inicial global de las ventanas ST.
    window.resize(800, 600)
    window.setAccessibleName(spec.get('titulo', 'Aplicación ST'))
    # Registro id -> QWidget: receive busca el mismo id recibido en cambiar_texto.
    widgets = {}
    count = 0

    def build(item, parent_layout, depth=0):
        """Solo admite widgets enumerados; no carga Python, HTML, .ui ni rutas externas."""
        nonlocal count
        count += 1
        if count > 64 or depth > 6:
            raise ValueError('Máximo 64 controles y 6 niveles')
        kind, name = item['tipo'], item['id']
        if name in widgets or not isinstance(name, str):
            raise ValueError('Identificador duplicado o inválido')
        text = str(item.get('texto', ''))
        # PUNTO DE EXTENSIÓN: agrega otra rama elif con un tipo declarativo nuevo.
        # Conecta señales a emit con id y datos JSON pequeños; no envíes secretos.
        if kind == 'etiqueta':
            widget = QtWidgets.QLabel(text)
            widget.setTextFormat(QtCore.Qt.TextFormat.PlainText)
            widget.setWordWrap(True)
        elif kind == 'boton':
            widget = QtWidgets.QPushButton(text)
            widget.clicked.connect(lambda checked=False: emit({'evento': 'click', 'id': name}))
        elif kind in {'entrada', 'clave'}:
            widget = QtWidgets.QLineEdit(text if kind == 'entrada' else '')
            if kind == 'clave':
                widget.setEchoMode(QtWidgets.QLineEdit.EchoMode.Password)
            else:
                widget.editingFinished.connect(lambda: emit({'evento': 'texto', 'id': name, 'texto': widget.text()}))
        # Texto multilínea se puede mostrar/modificar; hoy no emite cambios de texto.
        elif kind == 'texto':
            widget = QtWidgets.QPlainTextEdit(text)
        elif kind in {'casilla', 'opcion'}:
            widget = (QtWidgets.QCheckBox if kind == 'casilla' else QtWidgets.QRadioButton)(text)
            widget.toggled.connect(lambda checked: emit({'evento': 'cambio', 'id': name, 'valor': checked}))
        elif kind == 'lista':
            widget = QtWidgets.QListWidget()
            widget.addItems([str(value) for value in item.get('valores', [])][:100])
            widget.currentRowChanged.connect(lambda row: emit({'evento': 'seleccion', 'id': name, 'indice': row}))
        # La tabla recibe una instantánea; no sincroniza sus ediciones con ST.
        elif kind == 'tabla':
            rows = item.get('filas', [])[:100]
            columns = item.get('columnas', [])[:20]
            widget = QtWidgets.QTableWidget(len(rows), len(columns))
            widget.setHorizontalHeaderLabels(columns)
            for row, values in enumerate(rows):
                for column, value in enumerate(values[:len(columns)]):
                    widget.setItem(row, column, QtWidgets.QTableWidgetItem(str(value)))
        # La recursión comparte contador e ids: los límites incluyen controles hijos.
        elif kind == 'panel':
            widget = QtWidgets.QGroupBox(text)
            layout = QtWidgets.QVBoxLayout(widget)
            for child in item.get('hijos', []):
                build(child, layout, depth + 1)
        else:
            raise ValueError('Tipo de control no implementado')
        widget.setAccessibleName(str(item.get('accesible', text or name)))
        widgets[name] = widget
        parent_layout.addWidget(widget)

    layout = QtWidgets.QVBoxLayout(window)
    for item in spec.get('controles', []):
        build(item, layout)
    # PUNTO DE PERSONALIZACIÓN: paleta/espaciado Qt, independiente del CSS del IDE.
    if spec.get('tema', 'sistema') == 'oscuro':
        application.setStyleSheet('QWidget {background:#171717;color:#eeeeee;} '
                                 'QPushButton {padding:8px;border:1px solid #edee63;} '
                                 'QLineEdit,QPlainTextEdit {border:1px solid #666;}')
    # stdin no bloqueante + notifier integran órdenes con el bucle principal de Qt.
    # pending conserva una línea parcial entre lecturas y tiene un límite estricto.
    pending = bytearray()
    os.set_blocking(0, False)
    notifier = QtCore.QSocketNotifier(0, QtCore.QSocketNotifier.Type.Read)

    def receive(*unused):
        """Recibe cambios acotados sin bloquear el bucle de eventos gráfico."""
        try:
            data = os.read(0, 4096)
            if not data:
                application.quit()
                return
            pending.extend(data)
            if len(pending) > 8192:
                application.quit()
                return
            while b'\n' in pending:
                line, _, rest = pending.partition(b'\n')
                pending[:] = rest
                request = json.loads(line)
                # PUNTO DE EXTENSIÓN: nuevas órdenes deben tener emisor en Runtime.
                # Resuelve por id y valida clase/valores antes de modificar el widget.
                if request['operacion'] == 'cerrar':
                    application.quit()
                elif request['operacion'] == 'texto':
                    widget = widgets[request['id']]
                    if isinstance(widget, QtWidgets.QLineEdit) and widget.echoMode() == QtWidgets.QLineEdit.EchoMode.Password:
                        raise ValueError('No se modifica una clave mediante eventos')
                    if isinstance(widget, QtWidgets.QPlainTextEdit):
                        widget.setPlainText(request['texto'])
                    elif isinstance(widget, (QtWidgets.QLabel, QtWidgets.QPushButton, QtWidgets.QLineEdit)):
                        widget.setText(request['texto'])
                    else:
                        raise ValueError('Control sin propiedad texto')
        except Exception:
            emit({'evento': 'error', 'codigo': 'GUI_OPERACION'})

    # No crear un hilo para widgets: Qt exige manejarlos en este hilo principal.
    notifier.activated.connect(receive)
    window.show()
    emit({'evento': 'lista'})
    application.exec()
    emit({'evento': 'cerrada'})


try:
    main()
except Exception:
    emit({'evento': 'error', 'codigo': 'GUI_CONFIGURACION'})
    sys.exit(1)
