# Starship Setup

> See [starship docs](https://starship.cosmology.tech/) for detailed setup instructions.

**Setup**

Do this only once to create the Starship setup

```bash
make setup
```

**Start Infrastructure**

To start starship, use : 
```bash
make install
```

Once all nodes are up (see next section to check status) forward all the ports : 
```bash
make port-forward
```

**Check status**

To know the status of all the starship pods, use : 

```bash
kubectl get pods
```

Or for a watch version : 
```bash
make watch-pods
```

**Stop Infrastructure**
```bash
make stop
```

**Delete Infrastructure**

```bash
make clean-kind
```
