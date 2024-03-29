# Queue run

Manager task execution queues in parallel with a quenue depending on CPU/RAM load.


## Usage:

* Execute all tasks listed in `protein-calculation.bl` with up to three processes in parallel (`-j`). The backlog can be updated on the fly if you add the tasks at the end.

```shell
qrun -b protein-calculation.bl -j 3
```

* `qrun` accepts a directory with a multiple of backlogs.

```shell
qrun -b /tmp/backlog
```

* `qrun` can work in demon mode (`-d`). When tasks are added to the backlog, the tasks will be detected and executed.

```shell
qrun -b /tmp/backlog -j 5 -d
```
