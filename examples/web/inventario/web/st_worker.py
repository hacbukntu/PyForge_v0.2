"""Arranca únicamente el binario ST configurado, con límites; no es un sandbox del sistema."""
import os
import resource
import sys
# Estos límites no sustituyen un usuario de servicio sin permisos ni un contenedor.
resource.setrlimit(resource.RLIMIT_CPU, (2,2))
resource.setrlimit(resource.RLIMIT_AS, (128*1024*1024,128*1024*1024))
resource.setrlimit(resource.RLIMIT_FSIZE, (65536,65536))
resource.setrlimit(resource.RLIMIT_CORE, (0,0))
os.execve(sys.argv[1], [sys.argv[1]], {'LANG':'C.UTF-8'})
